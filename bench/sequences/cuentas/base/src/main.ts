import { run } from "./cli";
import { diskIo } from "./io";

const ran = run(process.argv.slice(2), diskIo());
process.stdout.write(`${ran.output}\n`);
process.exitCode = ran.code;
