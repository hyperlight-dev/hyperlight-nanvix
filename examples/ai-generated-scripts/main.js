/**
 * AI-Generated Scripts Example
 * 
 * Demonstrates running AI-generated JavaScript code in a Nanvix sandbox.
 * This is a critical use case for isolation - untrusted code from LLMs 
 * should never run directly in the host environment.
 */

require('dotenv').config();
const { NanvixSandbox } = require('hyperlight-nanvix');
const OpenAI = require('openai');
const fs = require('fs');
const path = require('path');

/**
 * Generate JavaScript code using OpenAI
 */
async function generateScript() {
    const apiKey = process.env.OPENAI_API_KEY;
    if (!apiKey) {
        throw new Error('OPENAI_API_KEY environment variable is required');
    }

    const client = new OpenAI({ apiKey });

    const prompts = [
        'Generate a JavaScript script that calculates the first 10 Fibonacci numbers and prints them. Use console.log.',
        'Generate a JavaScript script that calculates factorial of numbers from 1 to 10 and prints each result. Use console.log.',
        'Generate a JavaScript script that finds all prime numbers up to 50 and prints them. Use console.log.',
        'Generate a JavaScript script that reverses a string "Hello World" and counts vowels in it. Use console.log.',
        'Generate a JavaScript script that generates the multiplication table for numbers 1-5. Use console.log.',
        'Generate a JavaScript script that calculates the sum of squares of numbers from 1 to 10. Use console.log.',
        'Generate a JavaScript script that checks if numbers from 1 to 20 are even or odd and prints the results. Use console.log.',
        'Generate a JavaScript script that finds the greatest common divisor (GCD) of 48 and 18. Use console.log.',
        'Generate a JavaScript script that generates a simple pattern of asterisks (pyramid shape). Use console.log.',
        'Generate a JavaScript script that converts temperatures from Celsius to Fahrenheit for values 0, 10, 20, 30, 40. Use console.log.'
    ];

    const randomPrompt = prompts[Math.floor(Math.random() * prompts.length)];

    const completion = await client.chat.completions.create({
        model: "gpt-3.5-turbo",
        messages: [
            {
                role: "system",
                content: "You are a code generator. Generate only JavaScript code, no explanations or markdown. The code will run in QuickJS."
            },
            {
                role: "user",
                content: `${randomPrompt}

Requirements:
- Do not use any Node.js-specific APIs
- Return a final result
- Only return the code, no markdown, no explanations.`
            }
        ],
        temperature: 0.7,
        max_tokens: 500
    });

    let code = completion.choices[0].message.content;
    
    // Remove markdown code blocks if present (multiple patterns)
    code = code.replace(/```javascript\n?/g, '');
    code = code.replace(/```js\n?/g, '');
    code = code.replace(/```\n?/g, '');
    code = code.trim();
    
    return code;
}

async function main() {
    const count = parseInt(process.argv[2]) || 1;
    const tmpDir = '/tmp/hyperlight-nanvix-ai';
    
    if (!fs.existsSync(tmpDir)) {
        fs.mkdirSync(tmpDir, { recursive: true });
    }

    console.log(`Generating and executing ${count} AI-generated script(s)...\n`);

    for (let i = 0; i < count; i++) {
        const scriptPath = path.join(tmpDir, `ai-generated-${i}.js`);
        
        try {
            console.log(`[${i + 1}/${count}] Generating JavaScript code with AI...`);
            const generatedCode = await generateScript();
            
            console.log('Generated code:');
            console.log('-'.repeat(60));
            console.log(generatedCode);
            console.log('-'.repeat(60));
            
            fs.writeFileSync(scriptPath, generatedCode);
            
            console.log('Executing in Nanvix sandbox...');
            const sandbox = new NanvixSandbox({
                logDirectory: '/tmp/hyperlight-nanvix',
                tmpDirectory: '/tmp/hyperlight-nanvix'
            });
            
            const result = await sandbox.run(scriptPath);
            
            if (result.success) {
                console.log('Execution completed successfully\n');
            } else {
                console.error('Execution failed:', result.error);
                console.log('Continuing to next script...\n');
            }
            
        } catch (error) {
            console.error('Error:', error.message);
            process.exit(1);
        } finally {
            if (fs.existsSync(scriptPath)) {
                fs.unlinkSync(scriptPath);
            }
        }
    }
    
    console.log(`All ${count} script(s) completed successfully!`);
}

main();
