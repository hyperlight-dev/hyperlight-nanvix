console.log("Hello from JavaScript in Nanvix!");
console.log("Testing basic operations...");

// Test some basic operations
const data = { message: "Testing JSON", value: 42 };
console.log("JSON test:", JSON.stringify(data));

// Test arithmetic
for (let i = 1; i <= 5; i++) {
    console.log(`${i}! = ${factorial(i)}`);
}

function factorial(n) {
    return n <= 1 ? 1 : n * factorial(n - 1);
}

console.log("JavaScript execution completed!");