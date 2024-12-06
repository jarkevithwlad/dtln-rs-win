if (process.platform == "darwin") {
  module.exports = require("./index.node");
} else {
  module.exports = {};
}
