# Basic Ruby module inclusion patterns
# Tests include, extend, prepend statements

module Enumerable
  def each_with_index
  end
end

module Comparable
  def compare_to(other)
  end
end

module Serializable
  def to_json
  end
end

class MyCollection
  include Enumerable
  extend Comparable
  prepend Serializable

  def initialize
  end
end
