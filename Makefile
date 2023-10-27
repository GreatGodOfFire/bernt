EXE = bernt

ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
	COPY := copy target\release\bernt.exe
else
	NAME := $(EXE)
	COPY := cp target/release/bernt ./$(NAME)
endif

openbench:
	cargo build --release
	$(COPY)
