# My own tools

> *"What I cannot create, I do not understand."* \
> -- *Richard Feynman*

Project to learn by building my own version of existing tools/software.

## How to run
Two aliases in `fish/functions` were created to use it: \
`myown-update` to build all tools in release. \
`myown tool_name args` to use my own tools. (e.g. `echo "hello" | myown wc -c`)

## How to benchmark
Using `time` and `for` we can get an idea of the performances, for example: \
`time for i in (seq 1 1000); wc -l -c src/wc/test.txt; end > /dev/null` \
`time for i in (seq 1 1000); myown wc -l -c src/wc/test.txt; end > /dev/null`
