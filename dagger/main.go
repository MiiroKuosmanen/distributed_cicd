package main

import (
	"context"
	"fmt"
	"os"

	"dagger.io/dagger"
)

func main() {
	ctx := context.Background()

	// Connect to Dagger
	client, err := dagger.Connect(ctx, dagger.WithLogOutput(os.Stdout))
	if err != nil {
		panic(err)
	}
	defer client.Close()

	// Mount the source code from /root/client
	src := client.Host().Directory("/home/maso77/repos/distributed_cicd/client")
	shared := client.Host().Directory("/home/maso77/repos/distributed_cicd/shared")

	// Set up a build container (Modify base image as needed)
	build := client.Container().
			From("rust:latest"). // Use the Rust official image
			WithMountedDirectory("/app", src). // Mount Rust project directory
			WithMountedDirectory("/shared", shared). // Mount shared crate
			WithWorkdir("/app"). // Set working directory
			WithExec([]string{"cargo", "build", "--release"}) // Build the Rust appS


	// Run the build process
	output, err := build.Stdout(ctx)
	if err != nil {
		panic(err)
	}

	fmt.Println("Build Output:\n", output)
}
