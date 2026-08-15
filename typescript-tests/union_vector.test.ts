import assert from 'node:assert/strict';
import * as flatbuffers from 'flatbuffers';

import {
  CatT,
  DogT,
  Pet,
  TagT,
  Zoo,
  ZooT,
} from './generated/union_vector_generated.js';

function roundTrip(value: ZooT): ZooT {
  const builder = new flatbuffers.Builder();
  builder.finish(value.pack(builder));
  const buffer = new flatbuffers.ByteBuffer(builder.asUint8Array());
  return Zoo.getRootAsZoo(buffer).unpack();
}

const original = new ZooT(
  [Pet.Cat, Pet.NONE, Pet.Dog, Pet.Tag, Pet.Label],
  [new CatT('Milo'), null, new DogT(7), new TagT(42), 'hello'],
);

const unpacked = roundTrip(original);
assert.deepEqual(unpacked.petsType, [
  Pet.Cat,
  Pet.NONE,
  Pet.Dog,
  Pet.Tag,
  Pet.Label,
]);
assert.equal(unpacked.pets.length, 5);
assert.equal((unpacked.pets[0] as CatT).name, 'Milo');
assert.equal(unpacked.pets[1], null);
assert.equal((unpacked.pets[2] as DogT).age, 7);
assert.equal((unpacked.pets[3] as TagT).value, 42);
assert.equal(unpacked.pets[4], 'hello');

const repacked = roundTrip(unpacked);
assert.deepEqual(repacked.petsType, original.petsType);
assert.equal((repacked.pets[0] as CatT).name, 'Milo');
assert.equal(repacked.pets[1], null);
assert.equal((repacked.pets[2] as DogT).age, 7);
assert.equal((repacked.pets[3] as TagT).value, 42);
assert.equal(repacked.pets[4], 'hello');

assert.throws(
  () => roundTrip(new ZooT([Pet.Cat], [])),
  /must have equal lengths/,
);
assert.throws(
  () => roundTrip(new ZooT([Pet.NONE], [new CatT('invalid')])),
  /requires null for a NONE union element/,
);
assert.throws(
  () => roundTrip(new ZooT([Pet.Dog], [null])),
  /requires a value for a non-NONE union element/,
);
