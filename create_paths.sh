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

data_path=ba_data/$filename
tests_path=${data_path}/${filename}.${extension}.json
num_seconds=$((2 * 24 * 60 * 60))
num_paths=$((10 ** 7))

mkdir -p $data_path

set -x
sbatch -p single -n 40 -t 72:00:00 --job-name=${filename} --output=ba_data/${filename}_create_paths.txt create_paths -g $graph -n $num_paths -m $num_seconds -p $tests_path
set +x
