set -e

for count in 32 64 128 256 512 1024 2048 4096; do
    for size in 32 64 128 256 512 1024 2048 4096; do
        for queries in 32 64 128 256 512 2048 4096; do
            ../target/release/sadbench \
                -s test_data.csv \
                $count $size $queries
        done
    done
done
