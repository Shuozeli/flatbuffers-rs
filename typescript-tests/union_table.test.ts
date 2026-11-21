// Strict-mode compile + runtime checks for the generated TypeScript:
// object-API pack() of single union fields, create<Field>Vector for vectors of
// tables, bigint uint64 vectors, typed union accessors (no casts), the
// one-module-per-type layout with a cross-namespace name collision, and the
// --ts-flat-files layout.
import assert from 'node:assert/strict';
import * as flatbuffers from 'flatbuffers';

import { Game, Other } from './generated/modules/union_table.js';
import * as Flat from './generated/flat/union_table_generated.js';

const N = Game.Net;

function wrapperOf(bytes: Uint8Array): InstanceType<typeof N.Wrapper> {
  return N.Wrapper.getRootAsWrapper(new flatbuffers.ByteBuffer(bytes));
}

// 1. Object-API pack() of a table whose single union field holds a table that
//    itself holds a union and nested tables (previously threw "nested").
{
  // Arrange
  const command = new N.CommandT(
    7,
    N.Action.Equip,
    new N.EquipT(new N.ItemRefT(3, 0x8123_4567_89ab_cdefn)),
    [new N.ItemRefT(1, 1n), new N.ItemRefT(2, 2n)],
    [0xffff_ffff_ffff_fffen, 5n],
    new Other.ItemRefT('other'),
  );
  const wrapper = new N.WrapperT(N.AnyMsg.Command, command);
  const builder = new flatbuffers.Builder(64);

  // Act
  builder.finish(wrapper.pack(builder));
  const unpacked = wrapperOf(builder.asUint8Array()).unpack();

  // Assert
  assert.deepEqual(unpacked, wrapper);
}

// 2. String member of a single union field packs and unpacks.
{
  // Arrange
  const wrapper = new N.WrapperT(N.AnyMsg.Command, new N.CommandT(1, N.Action.Note, 'hello'));
  const builder = new flatbuffers.Builder(64);

  // Act
  builder.finish(wrapper.pack(builder));
  const command = wrapperOf(builder.asUint8Array()).msg(new N.Command());

  // Assert
  assert.equal(command?.actionType(), N.Action.Note);
  assert.equal(command?.action(''), 'hello');
  assert.equal((command?.unpack().action as string), 'hello');
}

// 3. create<Field>Vector for a vector of tables; typed union accessors.
{
  // Arrange
  const builder = new flatbuffers.Builder(64);
  const refs = [9, 10].map((slot) => {
    N.ItemRef.startItemRef(builder);
    N.ItemRef.addSlot(builder, slot);
    return N.ItemRef.endItemRef(builder);
  });
  const items = N.Command.createItemsVector(builder, refs);
  const ids = N.Command.createIdsVector(builder, [2n ** 64n - 1n]);
  N.Command.startCommand(builder);
  N.Command.addItems(builder, items);
  N.Command.addIds(builder, ids);
  const command = N.Command.endCommand(builder);
  builder.finish(N.Wrapper.createWrapper(builder, N.AnyMsg.Command, command));

  // Act
  const read = wrapperOf(builder.asUint8Array()).msg(new N.Command());

  // Assert: no casts needed, `read` is Command|null.
  assert.ok(read);
  assert.equal(read.itemsLength(), 2);
  assert.deepEqual([read.items(0)?.slot(), read.items(1)?.slot()], [9, 10]);
  const id: bigint | null = read.ids(0);
  assert.equal(id, 2n ** 64n - 1n);
  assert.equal(read.action(new N.Equip()), null);
}

// 4. uint64 vector element accessor of an absent vector is a bigint 0.
{
  // Arrange
  const builder = new flatbuffers.Builder(16);
  N.Command.startCommand(builder);
  builder.finish(N.Command.endCommand(builder));

  // Act
  const command = N.Command.getRootAsCommand(new flatbuffers.ByteBuffer(builder.asUint8Array()));

  // Assert
  assert.equal(command.ids(0), 0n);
  assert.equal(command.idsLength(), 0);
}

// 5. The --ts-flat-files layout produces the same bytes.
{
  // Arrange
  const F = Flat.Game.Net;
  const modular = new N.WrapperT(N.AnyMsg.Command, new N.CommandT(3, N.Action.Chat, new N.ChatT('hi')));
  const flat = new F.WrapperT(F.AnyMsg.Command, new F.CommandT(3, F.Action.Chat, new F.ChatT('hi')));
  const a = new flatbuffers.Builder(64);
  const b = new flatbuffers.Builder(64);

  // Act
  a.finish(modular.pack(a));
  b.finish(flat.pack(b));

  // Assert
  assert.deepEqual(Array.from(b.asUint8Array()), Array.from(a.asUint8Array()));
}

console.log('union_table: ok');
