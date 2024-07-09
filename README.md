# My own tools
Project to learn by building my own version of existing tools/software.

## How to run
Use `myown-update` to build all tools in release. \
Use `myown tool_name args` to use my own tools. (e.g. `echo "hello" | myown wc -c`) \
Check `myown-update` and `myown` functions in `fish/functions` folder. 

## How to benchmark
Using `time` and `for` we can get an idea of the performances, for example: \
`time for i in (seq 1 1000); wc -l -c src/wc/test.txt; end > /dev/null` \
`time for i in (seq 1 1000); myown wc -l -c src/wc/test.txt; end > /dev/null`
