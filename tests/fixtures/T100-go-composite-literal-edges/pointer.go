package main

func setup() {
    server := &Server{Port: 8080}
    client := &http.Client{Timeout: 30}
}
