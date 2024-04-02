set -ex
trap "exit" INT TERM
trap "kill 0" EXIT

cargo build --example algebra
BIN="./target/debug/examples/algebra"


PROCS=()

for i in $(seq 0 2)
do
if [ $i == 0 ]
then
    RUST_BACKTRACE=1  $BIN $i ./data/address &
    pid=$!
    PROCS[$i]=$pid
else
    $BIN $i ./data/address > /dev/null &
    pid=$!
    PROCS[$i]=$pid
fi
done

for pid in ${PROCS[@]}
do
wait $pid
done

echo done