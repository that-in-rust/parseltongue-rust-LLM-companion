# Qualified module inclusion patterns
# Tests namespace-qualified module names

module ActiveSupport
  module Callbacks
    def run_callbacks
    end
  end

  module Validations
    def validate
    end
  end
end

module Concerns
  module Cacheable
    def cache_key
    end
  end
end

class Model
  include ActiveSupport::Callbacks
  extend ActiveSupport::Validations
  prepend Concerns::Cacheable

  def save
  end
end
