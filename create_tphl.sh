#!/bin/bash

unset graph_path

while getopts :g: flag; do
  case "${flag}" in
  g) graph_path=${OPTARG} ;;
  esac
done

partition="dev_single -n 40"
time="-t 30"

graph_basename="$(basename -- "${graph_path}")"

mkdir ${graph_basename}

paths_path="${graph_basename}/${graph_basename}_paths.json"
tests_path="${graph_basename}/${graph_basename}_tests.json"
ch_path="${graph_basename}/${graph_basename}.di_ch_bincode"
hl_path="${graph_basename}/${graph_basename}.di_hl_bincode"

set -x
job_id_create_paths=$(
  sbatch -p ${partition} ${time} --job-name=${graph_basename}_create_paths \
    --output=${graph_basename}/${graph_basename}_create_paths.txt \
    --wrap=" \
      create_paths \
      --pathfinder ${graph_path} \
      --paths ${tests_path}"
)

job_id_create_tests=$(
  sbatch -p ${partition} ${time} --job-name=${graph_basename}_create_tests \
    --output=${graph_basename}/${graph_basename}_create_tests.txt \
    --wrap=" \
      create_tests \
      --graph ${graph_path} \
      --number-of-tests 10000 \
      --test-cases ${tests_path}"
)

job_id_create_top_down_hl=$(
  sbatch -p ${partition} ${time} --job-name=${graph_basename}_create_top_down_hl \
    --output=${graph_basename}/${graph_basename}_create_top_down_hl.txt \
    --dependency afterok:${job_id_create_paths} \
    --wrap=" \
      create_top_down_hl \
      --graph ${graph_path} \
      --paths ${tests_path} \
      --hub-graph ${hl_path}"
)

set +x
