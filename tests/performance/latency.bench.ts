interface BenchmarkResult {
  name: string;
  duration_ms: number;
  target_ms: number;
  passed: boolean;
}

async function benchmark(name: string, fn: () => Promise<void>, target_ms: number): Promise<BenchmarkResult> {
  const start = performance.now();
  await fn();
  const duration = performance.now() - start;

  return {
    name,
    duration_ms: Math.round(duration),
    target_ms,
    passed: duration <= target_ms,
  };
}

async function runBenchmarks() {
  console.log("Running Handy01 Performance Benchmarks\n");

  const results: BenchmarkResult[] = [];

  results.push(
    await benchmark("Correction style formatting", async () => {
      const styles = ["inline", "highlighted", "draft-final"] as const;
      for (const style of styles) {
        for (let i = 0; i < 1000; i++) {
          const original = "hello world";
          const corrected = "Hello world";
          let result: string;
          switch (style) {
            case "inline":
              result = corrected;
              break;
            case "highlighted":
              result = `[[${corrected}]]`;
              break;
            case "draft-final":
              result = `~~${original}~~ → ${corrected}`;
              break;
          }
        }
      }
    }, 10),
  );

  results.push(
    await benchmark("Text merge (1000 ops)", async () => {
      for (let i = 0; i < 1000; i++) {
        const existing = "Hello world how";
        const newText = "world how are you";
        const existingWords = existing.split(" ");
        const newWords = newText.split(" ");
        const result = [...new Set([...existingWords, ...newWords])].join(" ");
      }
    }, 50),
  );

  console.log("Results:");
  console.log("─".repeat(60));

  let allPassed = true;
  for (const result of results) {
    const status = result.passed ? "✓ PASS" : "✗ FAIL";
    console.log(
      `${status} | ${result.name.padEnd(30)} | ${result.duration_ms}ms (target: ${result.target_ms}ms)`,
    );
    if (!result.passed) allPassed = false;
  }

  console.log("─".repeat(60));
  console.log(allPassed ? "All benchmarks passed!" : "Some benchmarks failed!");

  if (!allPassed) {
    process.exit(1);
  }
}

runBenchmarks();
