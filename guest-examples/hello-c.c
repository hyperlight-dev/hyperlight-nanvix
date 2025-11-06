#include <stdio.h>

int main() {
    printf("Hello from C in Nanvix!\n");
    printf("Testing basic C operations...\n");
    
    // Test some basic arithmetic
    int a = 10, b = 20;
    int sum = a + b;
    printf("Sum: %d + %d = %d\n", a, b, sum);
    
    // Test a simple loop
    printf("Counting from 1 to 5:\n");
    for (int i = 1; i <= 5; i++) {
        printf("Count: %d\n", i);
    }
    
    printf("C execution completed!\n");
    return 0;
}