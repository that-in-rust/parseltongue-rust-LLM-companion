class Service
  def create
    user = User.new
    config = Config.new(debug: true)
  end
end
