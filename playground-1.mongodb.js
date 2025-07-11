/* global use, db */
// MongoDB Playground
// Use Ctrl+Space inside a snippet or a string literal to trigger completions.

const database = 'admin_db';
const collection = 'admin';

// Create a new database.
use(database);

// Create a new collection.
db.createCollection(collection);

// Insert admin user document
db.getCollection(collection).insertOne({
  _id: ObjectId("685258ebc25f4303af50ddf2"),
  username: "A001",
  password: "$argon2id$v=19$m=19456,t=2,p=1$GTwEdGQ07tZ1zOWLU8UShQ$5M3mYiVPgnR7nsH3rm7Orcdj24V8xGL+AZIHv1Uafwo"
});

// The prototype form to create a collection:
/* db.createCollection( <name>,
  {
    capped: <boolean>,
    autoIndexId: <boolean>,
    size: <number>,
    max: <number>,
    storageEngine: <document>,
    validator: <document>,
    validationLevel: <string>,
    validationAction: <string>,
    indexOptionDefaults: <document>,
    viewOn: <string>,
    pipeline: <pipeline>,
    collation: <document>,
    writeConcern: <document>,
    timeseries: { // Added in MongoDB 5.0
      timeField: <string>, // required for time series collections
      metaField: <string>,
      granularity: <string>,
      bucketMaxSpanSeconds: <number>, // Added in MongoDB 6.3
      bucketRoundingSeconds: <number>, // Added in MongoDB 6.3
    },
    expireAfterSeconds: <number>,
    clusteredIndex: <document>, // Added in MongoDB 5.3
  }
)*/

// More information on the `createCollection` command can be found at:
// https://www.mongodb.com/docs/manual/reference/method/db.createCollection/
