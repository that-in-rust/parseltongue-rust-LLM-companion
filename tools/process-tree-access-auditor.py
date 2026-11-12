"""Strict S3 process ancestry/argv audit and process-tree RSS measurement."""
import argparse
import ast
import json
import os
import re
import resource
import subprocess
import sys
import time
from pathlib import Path


def read_complete_trace_events(trace_source_text):
    pending_trace_event_rows = {}
    complete_trace_event_rows = []
    for trace_source_line_text in trace_source_text.splitlines():
        trace_line_match_value = re.match(r"^(?:\[pid\s+)?(\d+)\]?\s+(.*)$", trace_source_line_text)
        if not trace_line_match_value:
            continue
        process_identifier_number_value = int(trace_line_match_value[1])
        process_trace_event_text = trace_line_match_value[2]
        if "<unfinished ...>" in process_trace_event_text:
            pending_trace_event_rows[process_identifier_number_value] = process_trace_event_text.split("<unfinished ...>")[0]
            continue
        if process_trace_event_text.startswith("<... "):
            process_trace_event_text = pending_trace_event_rows.pop(process_identifier_number_value, "") + re.sub(
                r"^<\.\.\. \w+ resumed>", "", process_trace_event_text
            )
        complete_trace_event_rows.append((process_identifier_number_value, process_trace_event_text))
    return complete_trace_event_rows, pending_trace_event_rows


def check_rustc_discovery_arguments(command_argument_tail_values):
    if command_argument_tail_values == ["-vV"]:
        return True
    if command_argument_tail_values[:3] != ["-", "--crate-name", "___"]:
        return False
    discovery_print_kind_values = {"file-names", "sysroot", "split-debuginfo", "crate-name", "cfg"}
    discovery_print_flag_values = command_argument_tail_values[3:]
    selectors = []
    prints = []
    warning = False
    index = 0
    while index < len(discovery_print_flag_values):
        argument = discovery_print_flag_values[index]
        if argument == "--crate-type" and index + 1 < len(discovery_print_flag_values):
            selectors.append(discovery_print_flag_values[index + 1])
            index += 2
        elif argument.startswith("--print=") and argument.removeprefix("--print=") in discovery_print_kind_values:
            prints.append(argument)
            index += 1
        elif argument == "-Wwarnings" and not warning:
            warning = True
            index += 1
        else:
            return False
    return bool(prints) and len(prints) == len(set(prints)) and (
        not selectors and not warning or selectors == ["bin", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"] and warning
    )


def audit_process_trace_events(trace_source_text, mapper_process_identifier, mapper_executable_path=None):
    complete_trace_event_rows, pending_trace_event_rows = read_complete_trace_events(trace_source_text)
    process_parent_identifier_map = {}
    for process_identifier_number_value, process_trace_event_text in complete_trace_event_rows:
        child_process_match_value = re.match(r"(?:clone3?|fork|vfork)\(.*\)\s+=\s+(\d+)", process_trace_event_text)
        if child_process_match_value:
            process_parent_identifier_map[int(child_process_match_value[1])] = process_identifier_number_value
    process_command_context_map = {mapper_process_identifier: "mapper"}
    process_audit_violation_rows = []
    mapper_execution_seen_flag = False
    network_syscall_name_set = {
        "socket", "socketpair", "connect", "bind", "listen", "accept", "accept4",
        "send", "sendto", "sendmsg", "sendmmsg", "recv", "recvfrom", "recvmsg",
        "recvmmsg", "shutdown", "getsockname", "getpeername", "getsockopt", "setsockopt",
        "sockatmark",
    }

    def read_process_command_context(process_identifier_number_value):
        visited_process_identifier_values = set()
        while process_identifier_number_value not in visited_process_identifier_values:
            visited_process_identifier_values.add(process_identifier_number_value)
            if process_identifier_number_value in process_command_context_map:
                return process_command_context_map[process_identifier_number_value]
            process_identifier_number_value = process_parent_identifier_map.get(process_identifier_number_value)
            if process_identifier_number_value is None:
                return None
        return None

    def read_process_descriptor_annotation(process_trace_event_text):
        descriptor_match_value = re.match(r"[A-Za-z0-9_]+\(\s*\d+<([^>]*)>", process_trace_event_text)
        return descriptor_match_value[1] if descriptor_match_value else None

    for process_identifier_number_value, process_trace_event_text in complete_trace_event_rows:
        process_command_context_kind = read_process_command_context(process_identifier_number_value)
        if process_command_context_kind is None:
            trace_call_match_value = re.match(r"([A-Za-z0-9_]+)\(", process_trace_event_text)
            if (trace_call_match_value and trace_call_match_value[1] in network_syscall_name_set
                    or process_trace_event_text.startswith(("execve(", "execveat("))):
                process_audit_violation_rows.append(
                    f"{process_identifier_number_value}: unresolved process ancestry for syscall"
                )
            continue
        execution_at_flag = process_trace_event_text.startswith("execveat(")
        execution_event_flag = execution_at_flag or process_trace_event_text.startswith("execve(")
        if execution_event_flag:
            execution_argument_match_value = re.match(
                r'execve\(("(?:[^"\\]|\\.)*"),\s*(\[.*?\]),', process_trace_event_text
            )
            if execution_at_flag:
                execution_argument_match_value = re.match(
                    r'execveat\([^,]+,\s*("(?:[^"\\]|\\.)*"),\s*(\[.*?\]),',
                    process_trace_event_text,
                )
            if execution_argument_match_value is None:
                process_audit_violation_rows.append(
                    f"{process_identifier_number_value}: undecodable execution attempt"
                )
                continue
            try:
                command_executable_path_value = ast.literal_eval(execution_argument_match_value[1])
                command_argument_vector_values = ast.literal_eval(execution_argument_match_value[2])
            except (SyntaxError, ValueError):
                process_audit_violation_rows.append(f"{process_identifier_number_value}: undecodable exec argv")
                continue
            if (not isinstance(command_executable_path_value, str)
                    or not command_executable_path_value
                    or not isinstance(command_argument_vector_values, list)
                    or not command_argument_vector_values
                    or not all(isinstance(value, str) for value in command_argument_vector_values)):
                process_audit_violation_rows.append(f"{process_identifier_number_value}: undecodable exec argv")
                continue
            if execution_at_flag and not os.path.isabs(command_executable_path_value):
                process_audit_violation_rows.append(
                    f"{process_identifier_number_value}: unresolved execveat path {command_executable_path_value!r}"
                )
                continue
            execution_result_match_value = re.search(r"=\s*(-?\d+)(?:\s|$)", process_trace_event_text)
            if execution_result_match_value is None:
                process_audit_violation_rows.append(f"{process_identifier_number_value}: incomplete execution event")
                continue
            execution_succeeded_flag = int(execution_result_match_value[1]) == 0
            command_executable_short_name = os.path.basename(command_executable_path_value)
            command_argument_tail_values = command_argument_vector_values[1:]
            if process_identifier_number_value == mapper_process_identifier and not mapper_execution_seen_flag:
                mapper_execution_seen_flag = execution_succeeded_flag
                if (command_executable_path_value != str(mapper_executable_path) if mapper_executable_path is not None
                        else command_executable_short_name != "parseltongue"):
                    process_audit_violation_rows.append("audited root is not parseltongue")
                elif not execution_succeeded_flag:
                    process_audit_violation_rows.append("mapper root exec attempt failed")
                continue
            inherited_command_context_kind = process_command_context_kind
            if command_executable_short_name == "cargo" and command_argument_tail_values == ["metadata", "--offline"]:
                if inherited_command_context_kind == "mapper" or (
                    process_command_context_map.get(process_identifier_number_value) == "cargo"
                ):
                    if execution_succeeded_flag:
                        process_command_context_map[process_identifier_number_value] = "cargo"
                    continue
            elif command_executable_short_name == "rustc":
                if inherited_command_context_kind in {"mapper", "direct-rustc"} and command_argument_tail_values in [
                    ["--print", "cfg"], ["--print", "sysroot"]
                ]:
                    if execution_succeeded_flag:
                        process_command_context_map[process_identifier_number_value] = "direct-rustc"
                    continue
                if inherited_command_context_kind in {"cargo", "probe"} and check_rustc_discovery_arguments(command_argument_tail_values):
                    if execution_succeeded_flag:
                        process_command_context_map[process_identifier_number_value] = "probe"
                    continue
            violation_kind_value = "forbidden exec" if execution_succeeded_flag else "forbidden exec attempt"
            process_audit_violation_rows.append(
                f"{process_identifier_number_value}: {violation_kind_value} "
                f"{command_executable_path_value} {command_argument_tail_values!r}"
            )

        trace_call_match_value = re.match(r"([A-Za-z0-9_]+)\(", process_trace_event_text)
        if trace_call_match_value and trace_call_match_value[1] in network_syscall_name_set:
            network_syscall_name = trace_call_match_value[1]
            family_values = re.findall(r"\b(AF_[A-Z0-9_]+)\b", process_trace_event_text)
            descriptor_annotation_value = read_process_descriptor_annotation(
                process_trace_event_text
            )
            is_internet_endpoint = (
                any(value in {"AF_INET", "AF_INET6"} for value in family_values)
                or bool(re.search(r"\b(?:TCP|UDP):\[", process_trace_event_text))
                or bool(descriptor_annotation_value and re.search(
                    r"\b(?:TCP|UDP):\[", descriptor_annotation_value
                ))
            )
            if is_internet_endpoint:
                process_audit_violation_rows.append(
                    f"{process_identifier_number_value}: forbidden network syscall"
                )
            elif network_syscall_name in {"socket", "socketpair"}:
                if "AF_UNIX" not in family_values:
                    process_audit_violation_rows.append(
                        f"{process_identifier_number_value}: unresolved network socket family"
                    )
            else:
                descriptor_is_local = bool(
                    descriptor_annotation_value
                    and re.match(r"UNIX(?:[-:]|$)", descriptor_annotation_value)
                )
                has_local_address = (
                    "AF_UNIX" in family_values
                    and not (network_syscall_name in {"sendmsg", "sendmmsg"} and "msg_name=NULL" in process_trace_event_text)
                    and not (network_syscall_name in {"sendto", "sendmsg", "sendmmsg"}
                             and re.search(r"(?:msg_name=)?NULL", process_trace_event_text))
                )
                if descriptor_is_local or has_local_address:
                    continue
                else:
                    violation_kind_value = (
                        "unproven network send" if network_syscall_name in {
                            "send", "sendto", "sendmsg", "sendmmsg",
                        } else "unproven network syscall"
                    )
                    process_audit_violation_rows.append(
                        f"{process_identifier_number_value}: {violation_kind_value}"
                    )
        source_open_match_value = re.match(r'openat\([^,]+, ("(?:[^"\\]|\\.)*"), ([^)]*)\)', process_trace_event_text)
        if source_open_match_value and "O_WRONLY" not in source_open_match_value[2]:
            source_open_path_value = Path(ast.literal_eval(source_open_match_value[1]))
            if source_open_path_value.name != ".rustc_info.json" and ("target" in source_open_path_value.parts or "target" in source_open_path_value.resolve().parts):
                process_audit_violation_rows.append(f"{process_identifier_number_value}: forbidden target read {source_open_path_value}")
    if not mapper_execution_seen_flag:
        process_audit_violation_rows.append("mapper exec event missing; incomplete trace")
    for process_identifier_number_value in pending_trace_event_rows:
        if read_process_command_context(process_identifier_number_value) is not None:
            process_audit_violation_rows.append(f"{process_identifier_number_value}: unfinished trace event")
    return process_audit_violation_rows


def read_process_tree_residency(mapper_process_identifier_value, process_metadata_root_path=Path("/proc")):
    pending = [(mapper_process_identifier_value, None)]
    visited = set()
    resident_bytes = 0
    while pending:
        pid, expected_parent = pending.pop()
        if pid in visited:
            continue
        visited.add(pid)
        directory = process_metadata_root_path / str(pid)
        try:
            status = (directory / "status").read_text()
        except (FileNotFoundError, ProcessLookupError):
            continue
        parent = re.search(r"^PPid:\s+(\d+)", status, re.M)
        group = re.search(r"^Tgid:\s+(\d+)", status, re.M)
        if expected_parent is not None and (not parent or int(parent[1]) != expected_parent):
            continue
        if group and int(group[1]) != pid:
            continue
        resident = re.search(r"^VmRSS:\s+(\d+)\s+kB", status, re.M)
        resident_bytes += int(resident[1]) * 1024 if resident else 0
        try:
            tasks = list((directory / "task").iterdir())
        except (FileNotFoundError, ProcessLookupError):
            continue
        for task in tasks:
            try:
                children = (task / "children").read_text().split()
            except (FileNotFoundError, ProcessLookupError):
                continue
            pending.extend((int(child), pid) for child in children)
    return resident_bytes


def sample_process_tree_memory(command_argument_values, sample_output_file_path):
    if not command_argument_values:
        raise ValueError("nonempty command required")
    started = time.monotonic()
    usage = resource.getrusage(resource.RUSAGE_SELF)
    cpu_started = usage.ru_utime + usage.ru_stime
    process = subprocess.Popen(command_argument_values)
    samples = []
    try:
        while process.poll() is None:
            samples.append([time.monotonic() - started, read_process_tree_residency(process.pid)])
            time.sleep(.01)
    finally:
        if process.poll() is None:
            process.terminate()
            process.wait()
    exit_code = process.wait()
    usage = resource.getrusage(resource.RUSAGE_SELF)
    Path(sample_output_file_path).write_text(json.dumps({
        "mapper_process_identifier_value": process.pid,
        "mapper_process_exit_code": exit_code,
        "process_tree_peak_bytes": max((sample[1] for sample in samples), default=0),
        "process_tree_elapsed_seconds": time.monotonic() - started,
        "process_memory_sample_values": samples,
        "sample_interval_millisecond_value": 10,
        "sampler_process_cpu_seconds": usage.ru_utime + usage.ru_stime - cpu_started,
    }, indent=2) + "\n")
    return exit_code


def main():
    if sys.argv[1:2] == ["sample"]:
        parser = argparse.ArgumentParser()
        parser.add_argument("--memory-sample-output-path", required=True)
        parser.add_argument("command", nargs=argparse.REMAINDER)
        args = parser.parse_args(sys.argv[2:])
        command = args.command[1:] if args.command[:1] == ["--"] else args.command
        return sample_process_tree_memory(command, args.memory_sample_output_path)
    command_argument_parser_value = argparse.ArgumentParser()
    command_argument_parser_value.add_argument("--trace-source-input-path", required=True)
    command_argument_parser_value.add_argument("--mapper-process-identifier-value", required=True, type=int)
    command_argument_namespace_value = command_argument_parser_value.parse_args()
    process_audit_violation_rows = audit_process_trace_events(
        Path(command_argument_namespace_value.trace_source_input_path).read_text(),
        command_argument_namespace_value.mapper_process_identifier_value,
    )
    print(json.dumps({"process_audit_pass_flag": not process_audit_violation_rows,
                      "process_audit_violation_rows": process_audit_violation_rows}, indent=2))
    return int(bool(process_audit_violation_rows))


if __name__ == "__main__":
    sys.exit(main())
