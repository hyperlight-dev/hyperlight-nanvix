const { NanvixSandbox } = require('../index.js');

async function main() {
    console.log("Running guest-examples/hello.js...");
    
    try {
        const sandbox = new NanvixSandbox({
            logDirectory: '/tmp/hyperlight-nanvix',
            tmpDirectory: '/tmp/hyperlight-nanvix'
        });

        const result = await sandbox.run("guest-examples/hello.js");
        
        if (result.success) {
            console.log("Workload completed successfully!");
        } else {
            console.error("Error:", result.error);
            process.exit(1);
        }
    } catch (error) {
        console.error("Error:", error.message);
        process.exit(1);
    }
}

main();