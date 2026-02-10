package main

func start() {
    go func() {
        doWork()
    }()
}
