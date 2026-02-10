class Factory
  def build
    user = Models::User.new
    db = Database::Connection.new
  end
end
