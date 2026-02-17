def setup
  person = Person.new("John", 30)
  logger = Logger.new(level: :debug)
  server = Server.new(port: 8080, host: "localhost")
end
