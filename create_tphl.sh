#!/bin/bash

unset graph

while getopts :g:d: flag; do
	case "${flag}" in
	g) graph=${OPTARG} ;;
	esac
done

filename=$(basename -- "$graph")
extension="${filename##*.}"
filename="${filename%.*}"
job_type="create_tphl"

data_path=ba_data/$filename
tests_path=${data_path}/${filename}.${extension}.json
hub_graph=${data_path}/${filename}.${extension}.hl.bincode

mkdir -p $data_path

set -x
sbatch -p fat -n 80 -t 72:00:00 --job-name=${filename} \
	--output=ba_data/${filename}_${job_type}.txt --wrap="create_top_down_hl -g graphs/$graph -p $tests_path -h ${hub_graph}"
set +x
