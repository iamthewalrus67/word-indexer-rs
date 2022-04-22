#!/usr/bin/python3

import subprocess
import sys
import os
import platform
import configparser

PROGRAM_PATH = os.path.join("target", "release", "word_indexer_rs")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Not enough arguments", file=sys.stderr)
        exit(1)

    number_of_runs = 1
    try:
        number_of_runs = int(sys.argv[1])
    except:
        print("Specify the number of runs", file=sys.stderr)
        exit(2)

    # Compile program if not compiled
    if not os.path.isfile(PROGRAM_PATH):
        process = subprocess.Popen(["cargo", "build", "--release"])
        process.wait()

    args = [PROGRAM_PATH, sys.argv[2]]

    prev_res_a = ""
    prev_res_n = ""

    # Get output paths from config
    with open(sys.argv[2], 'r') as f:
        config_string = '[dummy_section]\n' + f.read()
    config = configparser.ConfigParser()
    config.read_string(config_string)
    res_a_path = config.get("dummy_section", "out_by_a").split()[0].replace('"', '')
    res_n_path = config.get("dummy_section", "out_by_n").split()[0].replace('"', '')

    min_output_time = [float('inf') for _ in range(4)]

    for iteration in range(number_of_runs):
        process = subprocess.Popen(args, stdout=subprocess.PIPE)
        process.wait()
        output_time = [int(i.decode('utf-8').split("=")[1]) for i in process.stdout.read().split()]

        min_output_time = output_time if min_output_time[0] > output_time[0] else min_output_time

        if iteration == 0:
            with open(res_n_path, 'r', encoding='latin-1') as f:
                prev_res_n = f.read()
            with open(res_a_path, 'r', encoding='latin-1') as f:
                prev_res_a = f.read()
        else:
            with open(res_n_path, 'r', encoding='latin-1') as f:
                current_res_n = f.read()
            with open(res_a_path, 'r', encoding='latin-1') as f:
                current_res_a = f.read()

            if current_res_a != prev_res_a or current_res_n != prev_res_n:
                print("Results are NOT the same", file=sys.stderr)
                exit(3)

    print(f"Total={min_output_time[0]}")
    print(f"Reading={min_output_time[1]}")
    print(f"Finding={min_output_time[2]}")
    print(f"Writing={min_output_time[3]}")
    print("Results are the same")
