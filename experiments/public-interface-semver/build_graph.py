"""A pinned semver/rustdoc-57 projection, deliberately not a general Rust resolver."""

import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
REV = "6ed8561154715b2c34df417a2052597d586f2c43"


def collect_public_interface_graph(doc, features):
    assert doc["format_version"] == 57 and doc["crate_version"] == "1.0.27"
    items = doc["index"]
    root = items[str(doc["root"])]
    nodes, edges, exports = {}, {}, {}

    def locate_source_item_span(item):
        span = item["span"]
        return dict(
            file=span["filename"].removeprefix("semver/"),
            begin=span["begin"],
            end=span["end"],
        )

    def add_public_graph_node(key, kind, item=None, **details):
        node = dict(id=key, kind=kind, **details)
        if item is not None:
            node["span"] = locate_source_item_span(item)
        if key in nodes:
            assert nodes[key] == node, key
        nodes[key] = node

    def add_typed_graph_edge(a, role, b, item, slot=None):
        edge = dict(source=a, role=role, target=b, span=locate_source_item_span(item))
        if slot is not None:
            edge["slot"] = slot
        key = (a, role, b, slot or "")
        assert key not in edges or edges[key] == edge, key
        edges[key] = edge

    def resolve_named_type_id(path):
        id_ = str(path["id"])
        if id_ in exports:
            return exports[id_]
        desc = doc["paths"][id_]
        assert desc["crate_id"] != 0, ("unaccounted local path", desc)
        name = "::".join(desc["path"])
        add_public_graph_node(name, "external_" + desc["kind"])
        return name

    def render_signature_type_refs(t, owner):
        if "generic" in t:
            assert t["generic"] == "Self"
            return "Self", [owner]
        if "primitive" in t:
            key = "primitive::" + t["primitive"]
            add_public_graph_node(key, "primitive")
            return t["primitive"], [key]
        if "borrowed_ref" in t:
            r = t["borrowed_ref"]
            s, refs = render_signature_type_refs(r["type"], owner)
            return "&" + ("mut " if r["is_mutable"] else "") + s, refs
        p = t["resolved_path"]
        key = resolve_named_type_id(p)
        refs, args = [key], []
        if p["args"]:
            params = p["args"]["angle_bracketed"]
            assert not params["constraints"]
            for arg in params["args"]:
                text, child = render_signature_type_refs(arg["type"], owner)
                refs.extend(child)
                args.append(text)
        return p["path"] + ("<" + ", ".join(args) + ">" if args else ""), refs

    def attach_signature_type_edges(t, owner, source, role, item, slot=None):
        text, refs = render_signature_type_refs(t, owner)
        for dest in set(refs):
            add_typed_graph_edge(source, role, dest, item, slot)
        return text

    add_public_graph_node("semver", "crate", root)
    for rid in root["inner"]["module"]["items"]:
        item = items[str(rid)]
        via = item["inner"].get("use")
        target = items[str(via["id"])] if via else item
        kind = next(iter(target["inner"]))
        assert item["visibility"] == "public" and kind in ("struct", "enum")
        name = "semver::" + (via["name"] if via else item["name"])
        exports[str(target["id"])] = name
        add_public_graph_node(name, kind, target)
        add_typed_graph_edge("semver", "reexports" if via else "exports", name, item)

    for rid, owner in exports.items():
        obj = items[rid]
        kind, data = next(iter(obj["inner"].items()))
        members = (
            data["kind"]["plain"]["fields"] if kind == "struct" else data["variants"]
        )
        for i in members:
            m = items[str(i)]
            key = owner + "::" + m["name"]
            if kind == "struct":
                assert m["visibility"] == "public"
                sig = attach_signature_type_edges(
                    m["inner"]["struct_field"], owner, key, "field_type", m
                )
                add_public_graph_node(key, "field", m, type=sig)
            else:
                assert m["inner"]["variant"]["kind"] == "plain"
                add_public_graph_node(key, "variant", m)
            add_typed_graph_edge(owner, "has_member", key, m)

        for i in data["impls"]:
            item = items[str(i)]
            impl = item["inner"]["impl"]
            if (
                not item["span"]
                or impl["is_synthetic"]
                or impl["blanket_impl"] is not None
            ):
                continue
            trait = impl["trait"]
            if trait:
                trait_id = resolve_named_type_id(trait)
                add_typed_graph_edge(owner, "implements", trait_id, item)
            for j in impl["items"]:
                m = items[str(j)]
                k, body = next(iter(m["inner"].items()))
                if trait:
                    if k != "assoc_type":
                        continue
                    key = f"<{owner} as {trait_id}>::{m['name']}"
                    sig = attach_signature_type_edges(
                        body["type"], owner, key, "associated_type", m
                    )
                    add_public_graph_node(key, "type_binding", m, type=sig)
                    add_typed_graph_edge(owner, "has_member", key, m)
                    continue
                if m["visibility"] != "public":
                    continue
                key = owner + "::" + m["name"]
                if k == "function":
                    sig = body["sig"]
                    params = []
                    for param, ty in sig["inputs"]:
                        if param == "self":
                            text, _ = render_signature_type_refs(ty, owner)
                        else:
                            text = attach_signature_type_edges(
                                ty, owner, key, "parameter", m, param
                            )
                        params.append([param, text])
                    output = (
                        attach_signature_type_edges(
                            sig["output"], owner, key, "returns", m
                        )
                        if sig["output"]
                        else "()"
                    )
                    add_public_graph_node(
                        key, "method", m, parameters=params, returns=output
                    )
                elif k == "assoc_const":
                    sig = attach_signature_type_edges(
                        body["type"], owner, key, "constant_type", m
                    )
                    add_public_graph_node(key, "constant", m, type=sig)
                else:
                    raise ValueError(k)
                add_typed_graph_edge(owner, "has_member", key, m)
    assert all(e["source"] in nodes and e["target"] in nodes for e in edges.values())
    return dict(
        schema=1,
        crate="semver",
        version="1.0.27",
        commit=REV,
        features=features,
        target=doc["target"]["triple"],
        rustdoc_format=doc["format_version"],
        nodes=[nodes[k] for k in sorted(nodes)],
        edges=[edges[k] for k in sorted(edges)],
    )


def score_blind_luna_proposals(graph, proposal):
    def short(key):
        return key.removeprefix("semver::")

    truth = {k: set() for k in ("types", "fields", "methods", "method_edges")}
    for n in graph["nodes"]:
        if n["kind"] in ("struct", "enum"):
            truth["types"].add((short(n["id"]),))
        if n["kind"] == "method":
            truth["methods"].add((short(n["id"]),))
    for e in graph["edges"]:
        src, role, dst = short(e["source"]), e["role"], short(e["target"])
        if (dst,) not in truth["types"]:
            continue
        if role == "field_type":
            truth["fields"].add((src, dst))
        if role in ("parameter", "returns"):
            truth["method_edges"].add((src, role, dst))
    a, b = proposal["fields_agent"], proposal["methods_agent"]
    guesses = dict(
        types={(x,) for x in a["types"]},
        fields=set(map(tuple, a["fields"])),
        methods={(x,) for x in b["methods"]},
        method_edges=set(map(tuple, b["edges"])),
    )
    return {
        k: dict(
            matched=len(truth[k] & guesses[k]),
            expected=len(truth[k]),
            proposed=len(guesses[k]),
            missing=sorted(truth[k] - guesses[k]),
            unexpected=sorted(guesses[k] - truth[k]),
        )
        for k in truth
    }


if __name__ == "__main__":
    repo = HERE / "work/semver"
    assert (
        subprocess.check_output(
            ["git", "-C", repo, "rev-parse", "HEAD"], text=True
        ).strip()
        == REV
    )
    subprocess.run(
        [
            "git",
            "-C",
            repo,
            "diff",
            "--quiet",
            "HEAD",
            "--",
            "src",
            "Cargo.toml",
            "build.rs",
        ],
        check=True,
    )
    for profile, features in [("default", ["std"]), ("serde", ["std", "serde"])]:
        data = json.loads((HERE / f"work/{profile}/doc/semver.json").read_text())
        graph = collect_public_interface_graph(data, features)
        (HERE / f"graph-{profile}.json").write_text(json.dumps(graph, indent=2) + "\n")
        if profile == "default":
            local_types = {
                n["id"] for n in graph["nodes"] if n["kind"] in {"struct", "enum"}
            }
            core = [
                e
                for e in graph["edges"]
                if e["target"] in local_types
                and e["role"] in {"parameter", "returns", "field_type"}
            ]
            (HERE / "local-signatures.json").write_text(
                json.dumps(core, indent=2) + "\n"
            )
            scores = score_blind_luna_proposals(
                graph, json.loads((HERE / "luna-proposals.json").read_text())
            )
            (HERE / "luna-scores.json").write_text(json.dumps(scores, indent=2) + "\n")
