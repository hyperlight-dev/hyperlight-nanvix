import asyncio
from hyperlight_nanvix import NanvixSandbox

async def main():
    print("Running guest-examples/hello.js...")
    
    try:
        sandbox = NanvixSandbox()
        result = await sandbox.run("guest-examples/hello.js")
        
        if result.success:
            print("Workload completed successfully!")
        else:
            print(f"Error: {result.error}")
            exit(1)
    except Exception as error:
        print(f"Error: {error}")
        exit(1)

asyncio.run(main())
