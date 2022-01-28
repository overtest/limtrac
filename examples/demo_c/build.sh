PREV_DIR=$(pwd)
EXEC_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

cd "$EXEC_DIR" || exit
mkdir -p "build"
cd "./build/" || exit
cmake ../
make
cd "$PREV_DIR" || exit