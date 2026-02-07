// v151 Bug Reproduction: Go Package Paths
// Go uses / for package paths and . for method access
// Go is LESS affected by :: issue but included for completeness

package main

import (
	"context"
	"fmt"
	"net/http"
	"database/sql"
	"encoding/json"
)

// UserService handles user operations
type UserService struct {
	db *sql.DB
}

// CreateUser creates a new user
func (s *UserService) CreateUser(ctx context.Context) (*User, error) {
	// Edge 1: Package function call
	fmt.Println("Creating user")

	// Edge 2: Nested package type
	req, _ := http.NewRequest("POST", "/users", nil)

	// Edge 3: Package constant/type
	var result sql.Result

	// Edge 4: Method on imported type
	_, _ = json.Marshal(req)

	user := &User{Name: "test"}
	s.processUser(ctx, user)
	return user, nil
}

func (s *UserService) processUser(ctx context.Context, user *User) {
	// Edge 5: Context method
	ctx.Done()

	// Edge 6: Nested package function
	http.StatusText(http.StatusOK)

	// Edge 7: fmt package call
	fmt.Printf("Processing: %s\n", user.Name)
}

// User represents a user entity
type User struct {
	Name string
}

func main() {
	service := &UserService{}
	_, _ = service.CreateUser(context.Background())
}
