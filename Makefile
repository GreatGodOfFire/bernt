EXE = Bernt

ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
else
	NAME := $(EXE)
endif

openbench:
	cargo build --release
	cp target/release/bernt ./$(EXE)
