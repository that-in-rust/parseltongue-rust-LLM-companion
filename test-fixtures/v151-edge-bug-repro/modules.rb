# v151 Bug Reproduction: Ruby Qualified Names with ::
# Ruby uses :: for module/class nesting
# Expected: Edges should be created, keys should be properly sanitized

module MyApp
  module Services
    class UserService
      def create_user
        # Edge 1: Qualified class instantiation
        user = ::MyApp::Models::User.new

        # Edge 2: Nested module constant access
        config = ::MyApp::Config::Settings::DATABASE

        # Edge 3: Top-level constant access
        logger = ::Logger.new(STDOUT)

        # Edge 4: Method call on qualified class
        ::MyApp::Utils::Validator.validate(user)

        process_user(user)
      end

      private

      def process_user(user)
        # Edge 5: Qualified class method call
        ::MyApp::Events::Publisher.publish("user.created", user)

        # Edge 6: Nested module method
        ::ActiveRecord::Base.transaction do
          user.save!
        end
      end
    end
  end

  module Models
    class User
      attr_accessor :name, :email
    end
  end

  module Utils
    class Validator
      def self.validate(obj)
        true
      end
    end
  end

  module Events
    class Publisher
      def self.publish(event, data)
        puts "Publishing: #{event}"
      end
    end
  end

  module Config
    module Settings
      DATABASE = "postgresql://localhost/myapp"
    end
  end
end
