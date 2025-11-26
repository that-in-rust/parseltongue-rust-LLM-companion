/* Simple calculator program to test parseltongue */
#include <stdio.h>
#include <stdlib.h>

/* Function prototypes */
int add(int a, int b);
int subtract(int a, int b);
int multiply(int a, int b);
float divide(int a, int b);
void print_menu();
int get_operation();

/* Add two numbers */
int add(int a, int b) {
    return a + b;
}

/* Subtract two numbers */
int subtract(int a, int b) {
    return a - b;
}

/* Multiply two numbers */
int multiply(int a, int b) {
    return a * b;
}

/* Divide two numbers */
float divide(int a, int b) {
    if (b == 0) {
        printf("Error: Division by zero!\n");
        return 0.0;
    }
    return (float)a / b;
}

/* Print the menu */
void print_menu() {
    printf("\n=== Simple Calculator ===\n");
    printf("1. Add\n");
    printf("2. Subtract\n");
    printf("3. Multiply\n");
    printf("4. Divide\n");
    printf("5. Exit\n");
    printf("========================\n");
}

/* Get user operation choice */
int get_operation() {
    int choice;
    printf("Enter your choice (1-5): ");
    scanf("%d", &choice);
    return choice;
}

int main() {
    int num1, num2;
    int choice;

    printf("Welcome to Simple Calculator!\n");

    while (1) {
        print_menu();
        choice = get_operation();

        if (choice == 5) {
            printf("Exiting calculator. Goodbye!\n");
            break;
        }

        if (choice < 1 || choice > 5) {
            printf("Invalid choice! Please try again.\n");
            continue;
        }

        printf("Enter first number: ");
        scanf("%d", &num1);
        printf("Enter second number: ");
        scanf("%d", &num2);

        switch (choice) {
            case 1:
                printf("Result: %d + %d = %d\n", num1, num2, add(num1, num2));
                break;
            case 2:
                printf("Result: %d - %d = %d\n", num1, num2, subtract(num1, num2));
                break;
            case 3:
                printf("Result: %d * %d = %d\n", num1, num2, multiply(num1, num2));
                break;
            case 4:
                printf("Result: %d / %d = %.2f\n", num1, num2, divide(num1, num2));
                break;
        }
    }

    return 0;
}
