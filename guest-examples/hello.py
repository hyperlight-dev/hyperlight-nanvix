#!/usr/bin/env python3

import json
import sys
import time

print("Hello from Python in Nanvix!")
print("Testing basic operations...")

# Test JSON operations
data = {"message": "Testing JSON", "value": 42, "python_version": sys.version.split()[0]}
print("JSON test:", json.dumps(data))

# Test arithmetic and functions
def factorial(n):
    return 1 if n <= 1 else n * factorial(n - 1)

print("Factorial calculations:")
for i in range(1, 6):
    print(f"{i}! = {factorial(i)}")

# Test basic math
print(f"Math: 2 + 3 = {2 + 3}")
print(f"Math: 10 ** 2 = {10 ** 2}")

# Test list operations
numbers = [1, 2, 3, 4, 5]
print(f"List sum: {sum(numbers)}")
print(f"List length: {len(numbers)}")

# Test string operations
text = "Nanvix"
print(f"String operations: '{text}' reversed is '{text[::-1]}'")

print("Python execution completed!")