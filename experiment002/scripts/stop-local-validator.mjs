import { stopValidatorFromPidFile } from "./lib/process.mjs";

const stopped = stopValidatorFromPidFile();
console.log(stopped ? "Stopped local validator from .local/validator.pid" : "No local validator pid file found");
