#!/bin/bash

function exec-fuzzer() {
  # parameters:
  #   fuzzer: should be either "lto" or "fast"
  #   timeout: in seconds
  #   cpu: which core to bind, default is 7
  fuzzer="${1}"
  timeout="${2}"
  declare -i cpu="${3}" || 7

  # last_update should look like this
  # [Stats #0] clients: 1, corpus: 425, objectives: 0, executions: 23597, exec/sec: 1511
  last_update=$(timeout "${timeout}" taskset -c "${cpu}" ../target/release/exercise-one-solution -c "${fuzzer}" | grep Stats | tail -1)

  # regex + cut below will return the total # of executions
  total_execs=$(echo $last_update | egrep -o "executions: ([0-9]+)" | cut -f2 -d' ')

  execs_per_sec=$((total_execs/"${timeout}"))

  echo $execs_per_sec
}

function average_of_five_runs() {
  # parameters:
  #   fuzzer: should be either "lto" or "fast"
  fuzzer="${1}"
  declare -i total_execs_per_sec=0
  declare -i total_runs=5
  timeout=120

  for i in $(seq 1 "${total_runs}");
  do
    current=$(exec-fuzzer "${fuzzer}" "${timeout}" $((i+1)))
    total_execs_per_sec=$((total_execs_per_sec+current))
    echo "[${fuzzer}][${i}] - ${current} execs/sec"
  done

  final=$((total_execs_per_sec/total_runs))
  echo "[${fuzzer}][avg] - ${final} execs/sec"
}

#average_of_five_runs fast
average_of_five_runs lto
