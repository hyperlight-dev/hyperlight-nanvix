// QuickJS file operations test
// This will trigger syscalls that our custom handler can intercept

import * as std from 'std';
import * as os from 'os';

console.log("Testing QuickJS file operations...");

try {
    // Test 1: Create a temporary file
    console.log("1. Creating temporary file...");
    const tmpFile = "/tmp/test_file.txt";
    const content = "Hello from QuickJS!";
    
    // Write to file (should trigger openat, write, close)
    std.out.puts("Writing to file: " + tmpFile);
    const file = std.open(tmpFile, "w");
    if (file) {
        file.puts(content);
        file.close();
        console.log("File written successfully");
    } else {
        console.log("Failed to open file for writing");
    }
    
    // Test 2: Read from file (should trigger openat, read, close)
    console.log("2. Reading from file...");
    const readFile = std.open(tmpFile, "r");
    if (readFile) {
        const readContent = readFile.readAsString();
        readFile.close();
        console.log("Read content:", readContent);
    } else {
        console.log("Failed to open file for reading");
    }
    
    // Test 3: Check file stats (should trigger stat syscalls)
    console.log("3. Checking file stats...");
    try {
        const stats = os.stat(tmpFile);
        console.log("File stats:", JSON.stringify(stats, null, 2));
    } catch (e) {
        console.log("Failed to stat file:", e);
    }
    
    // Test 4: Remove file (should trigger unlink)
    console.log("4. Removing file...");
    try {
        os.remove(tmpFile);
        console.log("File removed successfully");
    } catch (e) {
        console.log("Failed to remove file:", e);
    }

} catch (e) {
    console.log("Error during file operations:", e);
}

console.log("File operations test completed!");