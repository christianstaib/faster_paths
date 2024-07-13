#!/bin/bash

unset graph_path

while getopts :g: flag; do
  case "${flag}" in
  g) graph_path=${OPTARG} ;;
  esac
done

partition="dev_single -n 40"
# time="-t 72:00:00"
time="-t 30"

graph_basename="$(basename -- "${graph_path}")"

mkdir ${graph_basename}

paths_path="${graph_basename}/random_paths.json"
random_tests_path="${graph_basename}/random_tests.json"
random_tests_dijkstra_timing_results_path="${graph_basename}/random_tests_dijkstra_timing_results.json"
random_tests_ch_timing_results_path="${graph_basename}/random_tests_ch_timing_results.json"
random_tests_hl_timing_results_path="${graph_basename}/random_tests_hl_timing_results.json"
ch_path="${graph_basename}/graph.di_ch_bincode"
hl_path="${graph_basename}/graph.di_hl_bincode"

set -x
job_id_create_paths=$(
  sbatch -p ${partition} ${time} --job-name=${graph_basename}_create_paths \
    --output=${graph_basename}/${graph_basename}_create_paths.txt \
    --wrap=" \
      create_paths \
      --pathfinder ${graph_path} \
      --paths ${paths_path}" |
    grep -o '[0-9]\+'
)

job_id_create_tests=$(
  sbatch -p ${partition} ${time} --job-name=${graph_basename}_create_tests \
    --output=${graph_basename}/${graph_basename}_create_tests.txt \
    --wrap=" \
      create_tests \
      --graph ${graph_path} \
      --number-of-tests 10000 \
      --test-cases ${random_tests_path}" |
    grep -o '[0-9]\+'
)

job_id_create_top_down_hl=$(
  sbatch -p ${partition} ${time} --job-name=${graph_basename}_create_top_down_hl \
    --output=${graph_basename}/${graph_basename}_create_top_down_hl.txt \
    --dependency afterok:${job_id_create_paths} \
    --wrap=" \
      create_top_down_hl \
      --graph ${graph_path} \
      --paths ${paths_path} \
      --hub-graph ${hl_path}" |
    grep -o '[0-9]\+'
)

job_id_create_top_down_ch=$(
  sbatch -p ${partition} ${time} --job-name=${graph_basename}_create_top_down_ch \
    --output=${graph_basename}/${graph_basename}_create_top_down_ch.txt \
    --dependency afterok:${job_id_create_paths} \
    --wrap=" \
      create_top_down_ch \
      --graph ${graph_path} \
      --paths ${paths_path} \
      --contracted-graph ${ch_path}" |
    grep -o '[0-9]\+'
)

job_id_validate_and_time_dijkstra=$(
  sbatch -p ${partition} ${time} --job-name=${graph_basename}_validate_and_time_dijkstra \
    --output=${graph_basename}/${graph_basename}_validate_and_time_dijkstra.txt \
    --wrap=" \
      validate_and_time \
      --pathfinder ${graph_path} \
      --graph ${graph_path} \
      --test-cases ${random_tests_path} \
      --timing-results = ${random_tests_dijkstra_timing_results_path} \
      --maximum-number-of-tests 1000" |
    grep -o '[0-9]\+'
)

job_id_validate_and_time_ch=$(
  sbatch -p ${partition} ${time} --job-name=${graph_basename}_validate_and_time_ch \
    --output=${graph_basename}/${graph_basename}_validate_and_time_ch.txt \
    --dependency afterok:${job_id_create_top_down_ch}:${job_id_create_tests} \
    --wrap=" \
      validate_and_time \
      --pathfinder ${ch_path} \
      --graph ${graph_path} \
      --timing-results = ${random_tests_ch_timing_results_path} \
      --test-cases ${random_tests_path}" |
    grep -o '[0-9]\+'
)

job_id_validate_and_time_hl=$(
  sbatch -p ${partition} ${time} --job-name=${graph_basename}_validate_and_time_hl \
    --output=${graph_basename}/${graph_basename}_validate_and_time_hl.txt \
    --dependency afterok:${job_id_create_top_down_hl}:${job_id_create_tests} \
    --wrap=" \
      validate_and_time \
      --pathfinder ${hl_path} \
      --graph ${graph_path} \
      --timing-results = ${random_tests_hl_timing_results_path} \
      --test-cases ${random_tests_path}" |
    grep -o '[0-9]\+'
)

set +x
