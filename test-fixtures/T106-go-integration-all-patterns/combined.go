package main

type Server struct {
    Port int
}

func (s *Server) Start() {
    config := Config{Debug: true}
    go handleRequests()
    s.Port = 8080
}
