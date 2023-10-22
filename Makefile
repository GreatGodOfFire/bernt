EXE = bernt

ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
	COPY := copy target\release\$(NAME)
else
	NAME := $(EXE)
	COPY := cp target/release/$(NAME) ./$(NAME)
endif

openbench:
	cargo build --release
	$(COPY)
