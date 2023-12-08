#!/usr/bin/python3

import os
import re
import subprocess

RUNNING_RE = re.compile(r'^Running: (\S+)')
ANSWER_RE = re.compile(r'^Answer: (\S+)')
E_TIME_RE = re.compile(r'^Elapsed Time: (\S+)')

RESULTS_DIR = "results"
LATEST_FILE = os.path.join(RESULTS_DIR, "latest.txt")
LAST_FILE = os.path.join(RESULTS_DIR, "last.txt")
LATEST_CSV = os.path.join(RESULTS_DIR, "latest.csv")

class RunParser:
    def __init__(self):
        self.runs = []
        self._init_current()

    def _init_current(self):
        self.problem = None
        self.answer = None
        self.elapsed_time = None

    def add_line(self, line):
        m = RUNNING_RE.search(line)
        if m:
            self.problem = m.group(1)
            return
        
        m = ANSWER_RE.search(line)
        if m:
            self.answer = m.group(1)
            return
        
        m = E_TIME_RE.search(line)
        if m:
            self.elapsed_time = float(m.group(1))
            self.runs.append({
                'problem': self.problem,
                'answer': self.answer,
                'elapsed_time': self.elapsed_time
            })
            self._init_current()

    @staticmethod
    def parse(file_name):
        run_parser = RunParser()
        with open(file_name, "r") as fh:
            for line in fh:
                run_parser.add_line(line)
        return run_parser.runs

def to_csv_row(cells):
    return ",".join("" if cell is None else str(cell) for cell in cells)

def write_csv(runs, file_name):
    with open(file_name, "w") as fh:
        print(to_csv_row(["Problem", "ElapsedTime", "Answer"]), file=fh)
        for run in runs:
            print(to_csv_row([run['problem'], run['elapsed_time'], run['answer']]), file=fh)

def build():
    print("Building")
    r = subprocess.run(["cargo", "build", "--release"], capture_output=False)
    r.check_returncode()

def diff_results(latest_results, last_results):
    last_map = {}
    for result in last_results:
        last_map[result["problem"]] = result

    shown_diff_header = False

    def check_show_header():
        nonlocal shown_diff_header
        if not(shown_diff_header):
            print("--------------------------------------")
            print("Answer Differences:")
            print("--------------------------------------")
            shown_diff_header = True

    for result in latest_results:
        last_result = last_map.get(result["problem"])
        if last_result is None:
            check_show_header()
            print(f"New answer: {result['problem']} -> {result['answer']}")
        elif last_result['answer'] != result['answer']:
            check_show_header()
            print(f"Mismatch: {result['problem']} {last_result['problem']} != {result['problem']}")


def run():
    os.makedirs(RESULTS_DIR, exist_ok=True)

    with subprocess.Popen(
        ["cargo", "run", "--release"],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        encoding="UTF-8") as proc:
        
        run_parser = RunParser()

        with open(LATEST_FILE, "w") as out_fh:
            for line in proc.stdout:
                line = line.replace("\n", "")
                print(line)
                print(line, file=out_fh)
                run_parser.add_line(line)

        rc = proc.wait()
        if rc != 0:
            raise Exception(f"Exited with non zero exit code ({rc})")

        runs = run_parser.runs

        write_csv(runs, LATEST_CSV)

        if os.path.isfile(LAST_FILE):
            last_runs = RunParser.parse(LAST_FILE)
            diff_results(runs, last_runs)

        return

def main():
    build()
    run()

main()