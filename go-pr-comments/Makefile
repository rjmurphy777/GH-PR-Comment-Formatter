BINARY_NAME=pr-comments
LDFLAGS=-ldflags="-s -w"

.PHONY: build clean all linux windows darwin

build:
	go build $(LDFLAGS) -o $(BINARY_NAME) .

all: linux windows darwin

linux:
	GOOS=linux GOARCH=amd64 go build $(LDFLAGS) -o $(BINARY_NAME)-linux-amd64 .
	GOOS=linux GOARCH=arm64 go build $(LDFLAGS) -o $(BINARY_NAME)-linux-arm64 .

windows:
	GOOS=windows GOARCH=amd64 go build $(LDFLAGS) -o $(BINARY_NAME)-windows-amd64.exe .

darwin:
	GOOS=darwin GOARCH=amd64 go build $(LDFLAGS) -o $(BINARY_NAME)-darwin-amd64 .
	GOOS=darwin GOARCH=arm64 go build $(LDFLAGS) -o $(BINARY_NAME)-darwin-arm64 .

clean:
	rm -f $(BINARY_NAME) $(BINARY_NAME)-*

install: build
	cp $(BINARY_NAME) /usr/local/bin/
