import { $ } from 'bun';
import { unlink, writeFile } from 'fs/promises';
import { join } from 'path';

const PORT = process.env.PORT || 3001;
const FONT_PATH = process.env.FONT_PATH || './fonts';
const MAX_COMPILATION_TIME = 30000; // 30 seconds
const RATE_LIMIT_WINDOW = 60000; // 1 minute
const RATE_LIMIT_MAX = 10; // 10 compilations per minute

// Simple in-memory rate limiting
const rateLimitMap = new Map<string, number[]>();

function checkRateLimit(identifier: string): boolean {
  const now = Date.now();
  const requests = rateLimitMap.get(identifier) || [];
  
  // Remove old requests outside the window
  const recentRequests = requests.filter(time => now - time < RATE_LIMIT_WINDOW);
  
  if (recentRequests.length >= RATE_LIMIT_MAX) {
    return false;
  }
  
  recentRequests.push(now);
  rateLimitMap.set(identifier, recentRequests);
  return true;
}

async function compileTypst(code: string): Promise<Uint8Array> {
  const timestamp = Date.now();
  const tmpFile = `/tmp/resume-${timestamp}.typ`;
  const pdfFile = `/tmp/resume-${timestamp}.pdf`;

  try {
    // Write Typst code to temp file
    await writeFile(tmpFile, code);

    // Compile with timeout
    const compilePromise = $`typst compile ${tmpFile} ${pdfFile} --font-path ${FONT_PATH}`.quiet();
    const timeoutPromise = new Promise((_, reject) =>
      setTimeout(() => reject(new Error('Compilation timeout')), MAX_COMPILATION_TIME)
    );

    await Promise.race([compilePromise, timeoutPromise]);

    // Read compiled PDF
    const pdfData = await Bun.file(pdfFile).arrayBuffer();
    
    return new Uint8Array(pdfData);
  } finally {
    // Cleanup temp files
    try {
      await unlink(tmpFile);
      await unlink(pdfFile);
    } catch (e) {
      // Ignore cleanup errors
    }
  }
}

const server = Bun.serve({
  port: PORT,
  async fetch(req) {
    const url = new URL(req.url);

    // CORS headers
    const corsHeaders = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'POST, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type, Authorization',
    };

    // Handle preflight
    if (req.method === 'OPTIONS') {
      return new Response(null, { headers: corsHeaders });
    }

    // Health check
    if (url.pathname === '/health') {
      return new Response(JSON.stringify({ status: 'ok', timestamp: Date.now() }), {
        headers: { ...corsHeaders, 'Content-Type': 'application/json' }
      });
    }

    // Compilation endpoint
    if (req.method === 'POST' && url.pathname === '/compile') {
      try {
        // Optional: Verify auth token
        const auth = req.headers.get('Authorization');
        if (!auth?.startsWith('Bearer ')) {
          return new Response(JSON.stringify({ error: 'Unauthorized' }), {
            status: 401,
            headers: { ...corsHeaders, 'Content-Type': 'application/json' }
          });
        }

        // Rate limiting (use IP or token as identifier)
        const identifier = req.headers.get('X-Real-IP') || 'default';
        if (!checkRateLimit(identifier)) {
          return new Response(JSON.stringify({ error: 'Rate limit exceeded' }), {
            status: 429,
            headers: { ...corsHeaders, 'Content-Type': 'application/json' }
          });
        }

        // Parse request
        const body = await req.json();
        const { code } = body;

        if (!code || typeof code !== 'string') {
          return new Response(JSON.stringify({ error: 'Missing or invalid code' }), {
            status: 400,
            headers: { ...corsHeaders, 'Content-Type': 'application/json' }
          });
        }

        console.log(`[${new Date().toISOString()}] Compiling Typst code (${code.length} bytes)`);

        // Compile
        const pdf = await compileTypst(code);

        console.log(`[${new Date().toISOString()}] Compilation successful (${pdf.length} bytes)`);

        return new Response(pdf, {
          headers: {
            ...corsHeaders,
            'Content-Type': 'application/pdf',
            'Content-Length': pdf.length.toString(),
          }
        });
      } catch (error: any) {
        console.error(`[${new Date().toISOString()}] Compilation error:`, error.message);
        
        return new Response(JSON.stringify({ 
          error: 'Compilation failed',
          message: error.message 
        }), {
          status: 500,
          headers: { ...corsHeaders, 'Content-Type': 'application/json' }
        });
      }
    }

    return new Response('Not Found', { status: 404, headers: corsHeaders });
  },
});

console.log(`🚀 Typst compilation service running on http://localhost:${server.port}`);
console.log(`   Font path: ${FONT_PATH}`);
console.log(`   Max compilation time: ${MAX_COMPILATION_TIME}ms`);
console.log(`   Rate limit: ${RATE_LIMIT_MAX} requests per ${RATE_LIMIT_WINDOW}ms`);
