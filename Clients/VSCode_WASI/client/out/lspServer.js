"use strict";
/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */
Object.defineProperty(exports, "__esModule", { value: true });
exports.runServerProcess = void 0;
const node_1 = require("vscode-languageclient/node");
class ReadableStreamImpl {
    constructor(readable) {
        this.errorEmitter = new node_1.Emitter();
        this.closeEmitter = new node_1.Emitter();
        this.endEmitter = new node_1.Emitter();
        this.readable = readable;
    }
    get onData() {
        return this.readable.onData;
    }
    get onError() {
        return this.errorEmitter.event;
    }
    fireError(error, message, count) {
        this.errorEmitter.fire([error, message, count]);
    }
    get onClose() {
        return this.closeEmitter.event;
    }
    fireClose() {
        this.closeEmitter.fire(undefined);
    }
    onEnd(listener) {
        return this.endEmitter.event(listener);
    }
    fireEnd() {
        this.endEmitter.fire(undefined);
    }
}
class WritableStreamImpl {
    constructor(writable) {
        this.errorEmitter = new node_1.Emitter();
        this.closeEmitter = new node_1.Emitter();
        this.endEmitter = new node_1.Emitter();
        this.writable = writable;
    }
    get onError() {
        return this.errorEmitter.event;
    }
    fireError(error, message, count) {
        this.errorEmitter.fire([error, message, count]);
    }
    get onClose() {
        return this.closeEmitter.event;
    }
    fireClose() {
        this.closeEmitter.fire(undefined);
    }
    onEnd(listener) {
        return this.endEmitter.event(listener);
    }
    fireEnd() {
        this.endEmitter.fire(undefined);
    }
    write(data, _encoding) {
        if (typeof data === 'string') {
            return this.writable.write(data, 'utf-8');
        }
        else {
            return this.writable.write(data);
        }
    }
    end() {
    }
}
async function runServerProcess(process, readable = process.stdout, writable = process.stdin) {
    if (readable === undefined || writable === undefined) {
        throw new Error('Process created without streams or no streams provided.');
    }
    const reader = new ReadableStreamImpl(readable);
    const writer = new WritableStreamImpl(writable);
    process.run().then((value) => {
        if (value === 0) {
            reader.fireEnd();
        }
        else {
            reader.fireError([new Error(`Process exited with code: ${value}`), undefined, undefined]);
        }
    }, (error) => {
        reader.fireError([error, undefined, undefined]);
    });
    return { reader: new node_1.ReadableStreamMessageReader(reader), writer: new node_1.WriteableStreamMessageWriter(writer), detached: false };
}
exports.runServerProcess = runServerProcess;
//# sourceMappingURL=lspServer.js.map