"""Requalify M1 with one frozen parser-aligned executable."""
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parent.parent
sys.dont_write_bytecode=True
spec=importlib.util.spec_from_file_location("driver",ROOT/"tools/corpus-production-stage-measurements.py")
driver=importlib.util.module_from_spec(spec)
spec.loader.exec_module(driver)

def measure_parser_aligned_corpus(binary, evidence_name="parser-aligned-m1-measurements"):
    binary=Path(binary).resolve()
    digest=hashlib.sha256(binary.read_bytes()).hexdigest()
    pin=json.loads((ROOT/"docs/phase-20261111/parser-source-pin-manifest.json").read_text())["parser_source_commit_hash"]
    for record in json.loads((ROOT/"docs/phase-20261111/corpus-source-pin-manifest.json").read_text()):
        output=ROOT/record["corpus_evidence_directory_path"]/evidence_name
        output.mkdir(exist_ok=False)
        selected=dict(record,corpus_mapper_binary_path=str(binary),corpus_mapping_final_stage="syntax-spans",
                      corpus_evidence_directory_path=str(output))
        summary=driver.measure_corpus_production_stages(selected)
        summary["parser_executable_sha256_digest"]=digest
        summary["parser_source_commit_hash"]=pin
        (output/"production-stage-measurement-summary.json").write_text(json.dumps(summary,indent=2)+"\n")
        print(json.dumps(summary),flush=True)

if __name__=="__main__":
    measure_parser_aligned_corpus(sys.argv[1], *sys.argv[2:])
