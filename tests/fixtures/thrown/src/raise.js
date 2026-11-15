// What a library might throw: not always an `Error`.

/** Throws a string. */
function throwText() {
  throw "the disk is full";
}

/** Throws a plain object with a message. */
function throwObject() {
  throw { message: "quota exceeded", code: 507 };
}

/** Throws a number. */
function throwNumber() {
  throw 42;
}

/** Throws an `Error`. */
function throwError() {
  throw new RangeError("out of range");
}
