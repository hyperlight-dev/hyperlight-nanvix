#include <iostream>
#include <string>
#include <vector>

class Calculator {
private:
    std::string name;

public:
    Calculator(const std::string& name) : name(name) {
        std::cout << "Calculator '" << name << "' initialized!" << std::endl;
    }
    
    int add(int a, int b) {
        return a + b;
    }
    
    double multiply(double a, double b) {
        return a * b;
    }
    
    void showResults(const std::vector<int>& numbers) {
        std::cout << "Numbers: ";
        for (const auto& num : numbers) {
            std::cout << num << " ";
        }
        std::cout << std::endl;
    }
};

int main() {
    std::cout << "Hello from C++ in Nanvix!" << std::endl;
    std::cout << "Testing C++ features..." << std::endl;
    
    // Test class instantiation
    Calculator calc("NanvixCalc");
    
    // Test basic operations
    int sum = calc.add(15, 25);
    std::cout << "Addition: 15 + 25 = " << sum << std::endl;
    
    double product = calc.multiply(3.14, 2.0);
    std::cout << "Multiplication: 3.14 * 2.0 = " << product << std::endl;
    
    // Test STL vector
    std::vector<int> numbers = {1, 2, 3, 4, 5};
    calc.showResults(numbers);
    
    // Test string operations
    std::string message = "C++ execution completed successfully!";
    std::cout << message << std::endl;
    
    return 0;
}