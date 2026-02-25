"use strict";
(() => {
  // node_modules/d3-dispatch/src/dispatch.js
  var noop = { value: () => {
  } };
  function dispatch() {
    for (var i = 0, n = arguments.length, _ = {}, t; i < n; ++i) {
      if (!(t = arguments[i] + "") || t in _ || /[\s.]/.test(t)) throw new Error("illegal type: " + t);
      _[t] = [];
    }
    return new Dispatch(_);
  }
  function Dispatch(_) {
    this._ = _;
  }
  function parseTypenames(typenames, types) {
    return typenames.trim().split(/^|\s+/).map(function(t) {
      var name = "", i = t.indexOf(".");
      if (i >= 0) name = t.slice(i + 1), t = t.slice(0, i);
      if (t && !types.hasOwnProperty(t)) throw new Error("unknown type: " + t);
      return { type: t, name };
    });
  }
  Dispatch.prototype = dispatch.prototype = {
    constructor: Dispatch,
    on: function(typename, callback) {
      var _ = this._, T = parseTypenames(typename + "", _), t, i = -1, n = T.length;
      if (arguments.length < 2) {
        while (++i < n) if ((t = (typename = T[i]).type) && (t = get(_[t], typename.name))) return t;
        return;
      }
      if (callback != null && typeof callback !== "function") throw new Error("invalid callback: " + callback);
      while (++i < n) {
        if (t = (typename = T[i]).type) _[t] = set(_[t], typename.name, callback);
        else if (callback == null) for (t in _) _[t] = set(_[t], typename.name, null);
      }
      return this;
    },
    copy: function() {
      var copy = {}, _ = this._;
      for (var t in _) copy[t] = _[t].slice();
      return new Dispatch(copy);
    },
    call: function(type2, that) {
      if ((n = arguments.length - 2) > 0) for (var args = new Array(n), i = 0, n, t; i < n; ++i) args[i] = arguments[i + 2];
      if (!this._.hasOwnProperty(type2)) throw new Error("unknown type: " + type2);
      for (t = this._[type2], i = 0, n = t.length; i < n; ++i) t[i].value.apply(that, args);
    },
    apply: function(type2, that, args) {
      if (!this._.hasOwnProperty(type2)) throw new Error("unknown type: " + type2);
      for (var t = this._[type2], i = 0, n = t.length; i < n; ++i) t[i].value.apply(that, args);
    }
  };
  function get(type2, name) {
    for (var i = 0, n = type2.length, c2; i < n; ++i) {
      if ((c2 = type2[i]).name === name) {
        return c2.value;
      }
    }
  }
  function set(type2, name, callback) {
    for (var i = 0, n = type2.length; i < n; ++i) {
      if (type2[i].name === name) {
        type2[i] = noop, type2 = type2.slice(0, i).concat(type2.slice(i + 1));
        break;
      }
    }
    if (callback != null) type2.push({ name, value: callback });
    return type2;
  }
  var dispatch_default = dispatch;

  // node_modules/d3-selection/src/namespaces.js
  var xhtml = "http://www.w3.org/1999/xhtml";
  var namespaces_default = {
    svg: "http://www.w3.org/2000/svg",
    xhtml,
    xlink: "http://www.w3.org/1999/xlink",
    xml: "http://www.w3.org/XML/1998/namespace",
    xmlns: "http://www.w3.org/2000/xmlns/"
  };

  // node_modules/d3-selection/src/namespace.js
  function namespace_default(name) {
    var prefix = name += "", i = prefix.indexOf(":");
    if (i >= 0 && (prefix = name.slice(0, i)) !== "xmlns") name = name.slice(i + 1);
    return namespaces_default.hasOwnProperty(prefix) ? { space: namespaces_default[prefix], local: name } : name;
  }

  // node_modules/d3-selection/src/creator.js
  function creatorInherit(name) {
    return function() {
      var document2 = this.ownerDocument, uri = this.namespaceURI;
      return uri === xhtml && document2.documentElement.namespaceURI === xhtml ? document2.createElement(name) : document2.createElementNS(uri, name);
    };
  }
  function creatorFixed(fullname) {
    return function() {
      return this.ownerDocument.createElementNS(fullname.space, fullname.local);
    };
  }
  function creator_default(name) {
    var fullname = namespace_default(name);
    return (fullname.local ? creatorFixed : creatorInherit)(fullname);
  }

  // node_modules/d3-selection/src/selector.js
  function none() {
  }
  function selector_default(selector) {
    return selector == null ? none : function() {
      return this.querySelector(selector);
    };
  }

  // node_modules/d3-selection/src/selection/select.js
  function select_default(select) {
    if (typeof select !== "function") select = selector_default(select);
    for (var groups = this._groups, m2 = groups.length, subgroups = new Array(m2), j = 0; j < m2; ++j) {
      for (var group = groups[j], n = group.length, subgroup = subgroups[j] = new Array(n), node, subnode, i = 0; i < n; ++i) {
        if ((node = group[i]) && (subnode = select.call(node, node.__data__, i, group))) {
          if ("__data__" in node) subnode.__data__ = node.__data__;
          subgroup[i] = subnode;
        }
      }
    }
    return new Selection(subgroups, this._parents);
  }

  // node_modules/d3-selection/src/array.js
  function array(x3) {
    return x3 == null ? [] : Array.isArray(x3) ? x3 : Array.from(x3);
  }

  // node_modules/d3-selection/src/selectorAll.js
  function empty() {
    return [];
  }
  function selectorAll_default(selector) {
    return selector == null ? empty : function() {
      return this.querySelectorAll(selector);
    };
  }

  // node_modules/d3-selection/src/selection/selectAll.js
  function arrayAll(select) {
    return function() {
      return array(select.apply(this, arguments));
    };
  }
  function selectAll_default(select) {
    if (typeof select === "function") select = arrayAll(select);
    else select = selectorAll_default(select);
    for (var groups = this._groups, m2 = groups.length, subgroups = [], parents = [], j = 0; j < m2; ++j) {
      for (var group = groups[j], n = group.length, node, i = 0; i < n; ++i) {
        if (node = group[i]) {
          subgroups.push(select.call(node, node.__data__, i, group));
          parents.push(node);
        }
      }
    }
    return new Selection(subgroups, parents);
  }

  // node_modules/d3-selection/src/matcher.js
  function matcher_default(selector) {
    return function() {
      return this.matches(selector);
    };
  }
  function childMatcher(selector) {
    return function(node) {
      return node.matches(selector);
    };
  }

  // node_modules/d3-selection/src/selection/selectChild.js
  var find = Array.prototype.find;
  function childFind(match) {
    return function() {
      return find.call(this.children, match);
    };
  }
  function childFirst() {
    return this.firstElementChild;
  }
  function selectChild_default(match) {
    return this.select(match == null ? childFirst : childFind(typeof match === "function" ? match : childMatcher(match)));
  }

  // node_modules/d3-selection/src/selection/selectChildren.js
  var filter = Array.prototype.filter;
  function children() {
    return Array.from(this.children);
  }
  function childrenFilter(match) {
    return function() {
      return filter.call(this.children, match);
    };
  }
  function selectChildren_default(match) {
    return this.selectAll(match == null ? children : childrenFilter(typeof match === "function" ? match : childMatcher(match)));
  }

  // node_modules/d3-selection/src/selection/filter.js
  function filter_default(match) {
    if (typeof match !== "function") match = matcher_default(match);
    for (var groups = this._groups, m2 = groups.length, subgroups = new Array(m2), j = 0; j < m2; ++j) {
      for (var group = groups[j], n = group.length, subgroup = subgroups[j] = [], node, i = 0; i < n; ++i) {
        if ((node = group[i]) && match.call(node, node.__data__, i, group)) {
          subgroup.push(node);
        }
      }
    }
    return new Selection(subgroups, this._parents);
  }

  // node_modules/d3-selection/src/selection/sparse.js
  function sparse_default(update) {
    return new Array(update.length);
  }

  // node_modules/d3-selection/src/selection/enter.js
  function enter_default() {
    return new Selection(this._enter || this._groups.map(sparse_default), this._parents);
  }
  function EnterNode(parent, datum2) {
    this.ownerDocument = parent.ownerDocument;
    this.namespaceURI = parent.namespaceURI;
    this._next = null;
    this._parent = parent;
    this.__data__ = datum2;
  }
  EnterNode.prototype = {
    constructor: EnterNode,
    appendChild: function(child) {
      return this._parent.insertBefore(child, this._next);
    },
    insertBefore: function(child, next) {
      return this._parent.insertBefore(child, next);
    },
    querySelector: function(selector) {
      return this._parent.querySelector(selector);
    },
    querySelectorAll: function(selector) {
      return this._parent.querySelectorAll(selector);
    }
  };

  // node_modules/d3-selection/src/constant.js
  function constant_default(x3) {
    return function() {
      return x3;
    };
  }

  // node_modules/d3-selection/src/selection/data.js
  function bindIndex(parent, group, enter, update, exit, data) {
    var i = 0, node, groupLength = group.length, dataLength = data.length;
    for (; i < dataLength; ++i) {
      if (node = group[i]) {
        node.__data__ = data[i];
        update[i] = node;
      } else {
        enter[i] = new EnterNode(parent, data[i]);
      }
    }
    for (; i < groupLength; ++i) {
      if (node = group[i]) {
        exit[i] = node;
      }
    }
  }
  function bindKey(parent, group, enter, update, exit, data, key) {
    var i, node, nodeByKeyValue = /* @__PURE__ */ new Map(), groupLength = group.length, dataLength = data.length, keyValues = new Array(groupLength), keyValue;
    for (i = 0; i < groupLength; ++i) {
      if (node = group[i]) {
        keyValues[i] = keyValue = key.call(node, node.__data__, i, group) + "";
        if (nodeByKeyValue.has(keyValue)) {
          exit[i] = node;
        } else {
          nodeByKeyValue.set(keyValue, node);
        }
      }
    }
    for (i = 0; i < dataLength; ++i) {
      keyValue = key.call(parent, data[i], i, data) + "";
      if (node = nodeByKeyValue.get(keyValue)) {
        update[i] = node;
        node.__data__ = data[i];
        nodeByKeyValue.delete(keyValue);
      } else {
        enter[i] = new EnterNode(parent, data[i]);
      }
    }
    for (i = 0; i < groupLength; ++i) {
      if ((node = group[i]) && nodeByKeyValue.get(keyValues[i]) === node) {
        exit[i] = node;
      }
    }
  }
  function datum(node) {
    return node.__data__;
  }
  function data_default(value, key) {
    if (!arguments.length) return Array.from(this, datum);
    var bind = key ? bindKey : bindIndex, parents = this._parents, groups = this._groups;
    if (typeof value !== "function") value = constant_default(value);
    for (var m2 = groups.length, update = new Array(m2), enter = new Array(m2), exit = new Array(m2), j = 0; j < m2; ++j) {
      var parent = parents[j], group = groups[j], groupLength = group.length, data = arraylike(value.call(parent, parent && parent.__data__, j, parents)), dataLength = data.length, enterGroup = enter[j] = new Array(dataLength), updateGroup = update[j] = new Array(dataLength), exitGroup = exit[j] = new Array(groupLength);
      bind(parent, group, enterGroup, updateGroup, exitGroup, data, key);
      for (var i0 = 0, i1 = 0, previous, next; i0 < dataLength; ++i0) {
        if (previous = enterGroup[i0]) {
          if (i0 >= i1) i1 = i0 + 1;
          while (!(next = updateGroup[i1]) && ++i1 < dataLength) ;
          previous._next = next || null;
        }
      }
    }
    update = new Selection(update, parents);
    update._enter = enter;
    update._exit = exit;
    return update;
  }
  function arraylike(data) {
    return typeof data === "object" && "length" in data ? data : Array.from(data);
  }

  // node_modules/d3-selection/src/selection/exit.js
  function exit_default() {
    return new Selection(this._exit || this._groups.map(sparse_default), this._parents);
  }

  // node_modules/d3-selection/src/selection/join.js
  function join_default(onenter, onupdate, onexit) {
    var enter = this.enter(), update = this, exit = this.exit();
    if (typeof onenter === "function") {
      enter = onenter(enter);
      if (enter) enter = enter.selection();
    } else {
      enter = enter.append(onenter + "");
    }
    if (onupdate != null) {
      update = onupdate(update);
      if (update) update = update.selection();
    }
    if (onexit == null) exit.remove();
    else onexit(exit);
    return enter && update ? enter.merge(update).order() : update;
  }

  // node_modules/d3-selection/src/selection/merge.js
  function merge_default(context) {
    var selection2 = context.selection ? context.selection() : context;
    for (var groups0 = this._groups, groups1 = selection2._groups, m0 = groups0.length, m1 = groups1.length, m2 = Math.min(m0, m1), merges = new Array(m0), j = 0; j < m2; ++j) {
      for (var group0 = groups0[j], group1 = groups1[j], n = group0.length, merge = merges[j] = new Array(n), node, i = 0; i < n; ++i) {
        if (node = group0[i] || group1[i]) {
          merge[i] = node;
        }
      }
    }
    for (; j < m0; ++j) {
      merges[j] = groups0[j];
    }
    return new Selection(merges, this._parents);
  }

  // node_modules/d3-selection/src/selection/order.js
  function order_default() {
    for (var groups = this._groups, j = -1, m2 = groups.length; ++j < m2; ) {
      for (var group = groups[j], i = group.length - 1, next = group[i], node; --i >= 0; ) {
        if (node = group[i]) {
          if (next && node.compareDocumentPosition(next) ^ 4) next.parentNode.insertBefore(node, next);
          next = node;
        }
      }
    }
    return this;
  }

  // node_modules/d3-selection/src/selection/sort.js
  function sort_default(compare) {
    if (!compare) compare = ascending;
    function compareNode(a2, b) {
      return a2 && b ? compare(a2.__data__, b.__data__) : !a2 - !b;
    }
    for (var groups = this._groups, m2 = groups.length, sortgroups = new Array(m2), j = 0; j < m2; ++j) {
      for (var group = groups[j], n = group.length, sortgroup = sortgroups[j] = new Array(n), node, i = 0; i < n; ++i) {
        if (node = group[i]) {
          sortgroup[i] = node;
        }
      }
      sortgroup.sort(compareNode);
    }
    return new Selection(sortgroups, this._parents).order();
  }
  function ascending(a2, b) {
    return a2 < b ? -1 : a2 > b ? 1 : a2 >= b ? 0 : NaN;
  }

  // node_modules/d3-selection/src/selection/call.js
  function call_default() {
    var callback = arguments[0];
    arguments[0] = this;
    callback.apply(null, arguments);
    return this;
  }

  // node_modules/d3-selection/src/selection/nodes.js
  function nodes_default() {
    return Array.from(this);
  }

  // node_modules/d3-selection/src/selection/node.js
  function node_default() {
    for (var groups = this._groups, j = 0, m2 = groups.length; j < m2; ++j) {
      for (var group = groups[j], i = 0, n = group.length; i < n; ++i) {
        var node = group[i];
        if (node) return node;
      }
    }
    return null;
  }

  // node_modules/d3-selection/src/selection/size.js
  function size_default() {
    let size = 0;
    for (const node of this) ++size;
    return size;
  }

  // node_modules/d3-selection/src/selection/empty.js
  function empty_default() {
    return !this.node();
  }

  // node_modules/d3-selection/src/selection/each.js
  function each_default(callback) {
    for (var groups = this._groups, j = 0, m2 = groups.length; j < m2; ++j) {
      for (var group = groups[j], i = 0, n = group.length, node; i < n; ++i) {
        if (node = group[i]) callback.call(node, node.__data__, i, group);
      }
    }
    return this;
  }

  // node_modules/d3-selection/src/selection/attr.js
  function attrRemove(name) {
    return function() {
      this.removeAttribute(name);
    };
  }
  function attrRemoveNS(fullname) {
    return function() {
      this.removeAttributeNS(fullname.space, fullname.local);
    };
  }
  function attrConstant(name, value) {
    return function() {
      this.setAttribute(name, value);
    };
  }
  function attrConstantNS(fullname, value) {
    return function() {
      this.setAttributeNS(fullname.space, fullname.local, value);
    };
  }
  function attrFunction(name, value) {
    return function() {
      var v = value.apply(this, arguments);
      if (v == null) this.removeAttribute(name);
      else this.setAttribute(name, v);
    };
  }
  function attrFunctionNS(fullname, value) {
    return function() {
      var v = value.apply(this, arguments);
      if (v == null) this.removeAttributeNS(fullname.space, fullname.local);
      else this.setAttributeNS(fullname.space, fullname.local, v);
    };
  }
  function attr_default(name, value) {
    var fullname = namespace_default(name);
    if (arguments.length < 2) {
      var node = this.node();
      return fullname.local ? node.getAttributeNS(fullname.space, fullname.local) : node.getAttribute(fullname);
    }
    return this.each((value == null ? fullname.local ? attrRemoveNS : attrRemove : typeof value === "function" ? fullname.local ? attrFunctionNS : attrFunction : fullname.local ? attrConstantNS : attrConstant)(fullname, value));
  }

  // node_modules/d3-selection/src/window.js
  function window_default(node) {
    return node.ownerDocument && node.ownerDocument.defaultView || node.document && node || node.defaultView;
  }

  // node_modules/d3-selection/src/selection/style.js
  function styleRemove(name) {
    return function() {
      this.style.removeProperty(name);
    };
  }
  function styleConstant(name, value, priority) {
    return function() {
      this.style.setProperty(name, value, priority);
    };
  }
  function styleFunction(name, value, priority) {
    return function() {
      var v = value.apply(this, arguments);
      if (v == null) this.style.removeProperty(name);
      else this.style.setProperty(name, v, priority);
    };
  }
  function style_default(name, value, priority) {
    return arguments.length > 1 ? this.each((value == null ? styleRemove : typeof value === "function" ? styleFunction : styleConstant)(name, value, priority == null ? "" : priority)) : styleValue(this.node(), name);
  }
  function styleValue(node, name) {
    return node.style.getPropertyValue(name) || window_default(node).getComputedStyle(node, null).getPropertyValue(name);
  }

  // node_modules/d3-selection/src/selection/property.js
  function propertyRemove(name) {
    return function() {
      delete this[name];
    };
  }
  function propertyConstant(name, value) {
    return function() {
      this[name] = value;
    };
  }
  function propertyFunction(name, value) {
    return function() {
      var v = value.apply(this, arguments);
      if (v == null) delete this[name];
      else this[name] = v;
    };
  }
  function property_default(name, value) {
    return arguments.length > 1 ? this.each((value == null ? propertyRemove : typeof value === "function" ? propertyFunction : propertyConstant)(name, value)) : this.node()[name];
  }

  // node_modules/d3-selection/src/selection/classed.js
  function classArray(string) {
    return string.trim().split(/^|\s+/);
  }
  function classList(node) {
    return node.classList || new ClassList(node);
  }
  function ClassList(node) {
    this._node = node;
    this._names = classArray(node.getAttribute("class") || "");
  }
  ClassList.prototype = {
    add: function(name) {
      var i = this._names.indexOf(name);
      if (i < 0) {
        this._names.push(name);
        this._node.setAttribute("class", this._names.join(" "));
      }
    },
    remove: function(name) {
      var i = this._names.indexOf(name);
      if (i >= 0) {
        this._names.splice(i, 1);
        this._node.setAttribute("class", this._names.join(" "));
      }
    },
    contains: function(name) {
      return this._names.indexOf(name) >= 0;
    }
  };
  function classedAdd(node, names) {
    var list = classList(node), i = -1, n = names.length;
    while (++i < n) list.add(names[i]);
  }
  function classedRemove(node, names) {
    var list = classList(node), i = -1, n = names.length;
    while (++i < n) list.remove(names[i]);
  }
  function classedTrue(names) {
    return function() {
      classedAdd(this, names);
    };
  }
  function classedFalse(names) {
    return function() {
      classedRemove(this, names);
    };
  }
  function classedFunction(names, value) {
    return function() {
      (value.apply(this, arguments) ? classedAdd : classedRemove)(this, names);
    };
  }
  function classed_default(name, value) {
    var names = classArray(name + "");
    if (arguments.length < 2) {
      var list = classList(this.node()), i = -1, n = names.length;
      while (++i < n) if (!list.contains(names[i])) return false;
      return true;
    }
    return this.each((typeof value === "function" ? classedFunction : value ? classedTrue : classedFalse)(names, value));
  }

  // node_modules/d3-selection/src/selection/text.js
  function textRemove() {
    this.textContent = "";
  }
  function textConstant(value) {
    return function() {
      this.textContent = value;
    };
  }
  function textFunction(value) {
    return function() {
      var v = value.apply(this, arguments);
      this.textContent = v == null ? "" : v;
    };
  }
  function text_default(value) {
    return arguments.length ? this.each(value == null ? textRemove : (typeof value === "function" ? textFunction : textConstant)(value)) : this.node().textContent;
  }

  // node_modules/d3-selection/src/selection/html.js
  function htmlRemove() {
    this.innerHTML = "";
  }
  function htmlConstant(value) {
    return function() {
      this.innerHTML = value;
    };
  }
  function htmlFunction(value) {
    return function() {
      var v = value.apply(this, arguments);
      this.innerHTML = v == null ? "" : v;
    };
  }
  function html_default(value) {
    return arguments.length ? this.each(value == null ? htmlRemove : (typeof value === "function" ? htmlFunction : htmlConstant)(value)) : this.node().innerHTML;
  }

  // node_modules/d3-selection/src/selection/raise.js
  function raise() {
    if (this.nextSibling) this.parentNode.appendChild(this);
  }
  function raise_default() {
    return this.each(raise);
  }

  // node_modules/d3-selection/src/selection/lower.js
  function lower() {
    if (this.previousSibling) this.parentNode.insertBefore(this, this.parentNode.firstChild);
  }
  function lower_default() {
    return this.each(lower);
  }

  // node_modules/d3-selection/src/selection/append.js
  function append_default(name) {
    var create2 = typeof name === "function" ? name : creator_default(name);
    return this.select(function() {
      return this.appendChild(create2.apply(this, arguments));
    });
  }

  // node_modules/d3-selection/src/selection/insert.js
  function constantNull() {
    return null;
  }
  function insert_default(name, before) {
    var create2 = typeof name === "function" ? name : creator_default(name), select = before == null ? constantNull : typeof before === "function" ? before : selector_default(before);
    return this.select(function() {
      return this.insertBefore(create2.apply(this, arguments), select.apply(this, arguments) || null);
    });
  }

  // node_modules/d3-selection/src/selection/remove.js
  function remove() {
    var parent = this.parentNode;
    if (parent) parent.removeChild(this);
  }
  function remove_default() {
    return this.each(remove);
  }

  // node_modules/d3-selection/src/selection/clone.js
  function selection_cloneShallow() {
    var clone = this.cloneNode(false), parent = this.parentNode;
    return parent ? parent.insertBefore(clone, this.nextSibling) : clone;
  }
  function selection_cloneDeep() {
    var clone = this.cloneNode(true), parent = this.parentNode;
    return parent ? parent.insertBefore(clone, this.nextSibling) : clone;
  }
  function clone_default(deep) {
    return this.select(deep ? selection_cloneDeep : selection_cloneShallow);
  }

  // node_modules/d3-selection/src/selection/datum.js
  function datum_default(value) {
    return arguments.length ? this.property("__data__", value) : this.node().__data__;
  }

  // node_modules/d3-selection/src/selection/on.js
  function contextListener(listener) {
    return function(event) {
      listener.call(this, event, this.__data__);
    };
  }
  function parseTypenames2(typenames) {
    return typenames.trim().split(/^|\s+/).map(function(t) {
      var name = "", i = t.indexOf(".");
      if (i >= 0) name = t.slice(i + 1), t = t.slice(0, i);
      return { type: t, name };
    });
  }
  function onRemove(typename) {
    return function() {
      var on = this.__on;
      if (!on) return;
      for (var j = 0, i = -1, m2 = on.length, o; j < m2; ++j) {
        if (o = on[j], (!typename.type || o.type === typename.type) && o.name === typename.name) {
          this.removeEventListener(o.type, o.listener, o.options);
        } else {
          on[++i] = o;
        }
      }
      if (++i) on.length = i;
      else delete this.__on;
    };
  }
  function onAdd(typename, value, options) {
    return function() {
      var on = this.__on, o, listener = contextListener(value);
      if (on) for (var j = 0, m2 = on.length; j < m2; ++j) {
        if ((o = on[j]).type === typename.type && o.name === typename.name) {
          this.removeEventListener(o.type, o.listener, o.options);
          this.addEventListener(o.type, o.listener = listener, o.options = options);
          o.value = value;
          return;
        }
      }
      this.addEventListener(typename.type, listener, options);
      o = { type: typename.type, name: typename.name, value, listener, options };
      if (!on) this.__on = [o];
      else on.push(o);
    };
  }
  function on_default(typename, value, options) {
    var typenames = parseTypenames2(typename + ""), i, n = typenames.length, t;
    if (arguments.length < 2) {
      var on = this.node().__on;
      if (on) for (var j = 0, m2 = on.length, o; j < m2; ++j) {
        for (i = 0, o = on[j]; i < n; ++i) {
          if ((t = typenames[i]).type === o.type && t.name === o.name) {
            return o.value;
          }
        }
      }
      return;
    }
    on = value ? onAdd : onRemove;
    for (i = 0; i < n; ++i) this.each(on(typenames[i], value, options));
    return this;
  }

  // node_modules/d3-selection/src/selection/dispatch.js
  function dispatchEvent(node, type2, params) {
    var window2 = window_default(node), event = window2.CustomEvent;
    if (typeof event === "function") {
      event = new event(type2, params);
    } else {
      event = window2.document.createEvent("Event");
      if (params) event.initEvent(type2, params.bubbles, params.cancelable), event.detail = params.detail;
      else event.initEvent(type2, false, false);
    }
    node.dispatchEvent(event);
  }
  function dispatchConstant(type2, params) {
    return function() {
      return dispatchEvent(this, type2, params);
    };
  }
  function dispatchFunction(type2, params) {
    return function() {
      return dispatchEvent(this, type2, params.apply(this, arguments));
    };
  }
  function dispatch_default2(type2, params) {
    return this.each((typeof params === "function" ? dispatchFunction : dispatchConstant)(type2, params));
  }

  // node_modules/d3-selection/src/selection/iterator.js
  function* iterator_default() {
    for (var groups = this._groups, j = 0, m2 = groups.length; j < m2; ++j) {
      for (var group = groups[j], i = 0, n = group.length, node; i < n; ++i) {
        if (node = group[i]) yield node;
      }
    }
  }

  // node_modules/d3-selection/src/selection/index.js
  var root = [null];
  function Selection(groups, parents) {
    this._groups = groups;
    this._parents = parents;
  }
  function selection() {
    return new Selection([[document.documentElement]], root);
  }
  function selection_selection() {
    return this;
  }
  Selection.prototype = selection.prototype = {
    constructor: Selection,
    select: select_default,
    selectAll: selectAll_default,
    selectChild: selectChild_default,
    selectChildren: selectChildren_default,
    filter: filter_default,
    data: data_default,
    enter: enter_default,
    exit: exit_default,
    join: join_default,
    merge: merge_default,
    selection: selection_selection,
    order: order_default,
    sort: sort_default,
    call: call_default,
    nodes: nodes_default,
    node: node_default,
    size: size_default,
    empty: empty_default,
    each: each_default,
    attr: attr_default,
    style: style_default,
    property: property_default,
    classed: classed_default,
    text: text_default,
    html: html_default,
    raise: raise_default,
    lower: lower_default,
    append: append_default,
    insert: insert_default,
    remove: remove_default,
    clone: clone_default,
    datum: datum_default,
    on: on_default,
    dispatch: dispatch_default2,
    [Symbol.iterator]: iterator_default
  };
  var selection_default = selection;

  // node_modules/d3-color/src/define.js
  function define_default(constructor, factory, prototype) {
    constructor.prototype = factory.prototype = prototype;
    prototype.constructor = constructor;
  }
  function extend(parent, definition) {
    var prototype = Object.create(parent.prototype);
    for (var key in definition) prototype[key] = definition[key];
    return prototype;
  }

  // node_modules/d3-color/src/color.js
  function Color() {
  }
  var darker = 0.7;
  var brighter = 1 / darker;
  var reI = "\\s*([+-]?\\d+)\\s*";
  var reN = "\\s*([+-]?(?:\\d*\\.)?\\d+(?:[eE][+-]?\\d+)?)\\s*";
  var reP = "\\s*([+-]?(?:\\d*\\.)?\\d+(?:[eE][+-]?\\d+)?)%\\s*";
  var reHex = /^#([0-9a-f]{3,8})$/;
  var reRgbInteger = new RegExp(`^rgb\\(${reI},${reI},${reI}\\)$`);
  var reRgbPercent = new RegExp(`^rgb\\(${reP},${reP},${reP}\\)$`);
  var reRgbaInteger = new RegExp(`^rgba\\(${reI},${reI},${reI},${reN}\\)$`);
  var reRgbaPercent = new RegExp(`^rgba\\(${reP},${reP},${reP},${reN}\\)$`);
  var reHslPercent = new RegExp(`^hsl\\(${reN},${reP},${reP}\\)$`);
  var reHslaPercent = new RegExp(`^hsla\\(${reN},${reP},${reP},${reN}\\)$`);
  var named = {
    aliceblue: 15792383,
    antiquewhite: 16444375,
    aqua: 65535,
    aquamarine: 8388564,
    azure: 15794175,
    beige: 16119260,
    bisque: 16770244,
    black: 0,
    blanchedalmond: 16772045,
    blue: 255,
    blueviolet: 9055202,
    brown: 10824234,
    burlywood: 14596231,
    cadetblue: 6266528,
    chartreuse: 8388352,
    chocolate: 13789470,
    coral: 16744272,
    cornflowerblue: 6591981,
    cornsilk: 16775388,
    crimson: 14423100,
    cyan: 65535,
    darkblue: 139,
    darkcyan: 35723,
    darkgoldenrod: 12092939,
    darkgray: 11119017,
    darkgreen: 25600,
    darkgrey: 11119017,
    darkkhaki: 12433259,
    darkmagenta: 9109643,
    darkolivegreen: 5597999,
    darkorange: 16747520,
    darkorchid: 10040012,
    darkred: 9109504,
    darksalmon: 15308410,
    darkseagreen: 9419919,
    darkslateblue: 4734347,
    darkslategray: 3100495,
    darkslategrey: 3100495,
    darkturquoise: 52945,
    darkviolet: 9699539,
    deeppink: 16716947,
    deepskyblue: 49151,
    dimgray: 6908265,
    dimgrey: 6908265,
    dodgerblue: 2003199,
    firebrick: 11674146,
    floralwhite: 16775920,
    forestgreen: 2263842,
    fuchsia: 16711935,
    gainsboro: 14474460,
    ghostwhite: 16316671,
    gold: 16766720,
    goldenrod: 14329120,
    gray: 8421504,
    green: 32768,
    greenyellow: 11403055,
    grey: 8421504,
    honeydew: 15794160,
    hotpink: 16738740,
    indianred: 13458524,
    indigo: 4915330,
    ivory: 16777200,
    khaki: 15787660,
    lavender: 15132410,
    lavenderblush: 16773365,
    lawngreen: 8190976,
    lemonchiffon: 16775885,
    lightblue: 11393254,
    lightcoral: 15761536,
    lightcyan: 14745599,
    lightgoldenrodyellow: 16448210,
    lightgray: 13882323,
    lightgreen: 9498256,
    lightgrey: 13882323,
    lightpink: 16758465,
    lightsalmon: 16752762,
    lightseagreen: 2142890,
    lightskyblue: 8900346,
    lightslategray: 7833753,
    lightslategrey: 7833753,
    lightsteelblue: 11584734,
    lightyellow: 16777184,
    lime: 65280,
    limegreen: 3329330,
    linen: 16445670,
    magenta: 16711935,
    maroon: 8388608,
    mediumaquamarine: 6737322,
    mediumblue: 205,
    mediumorchid: 12211667,
    mediumpurple: 9662683,
    mediumseagreen: 3978097,
    mediumslateblue: 8087790,
    mediumspringgreen: 64154,
    mediumturquoise: 4772300,
    mediumvioletred: 13047173,
    midnightblue: 1644912,
    mintcream: 16121850,
    mistyrose: 16770273,
    moccasin: 16770229,
    navajowhite: 16768685,
    navy: 128,
    oldlace: 16643558,
    olive: 8421376,
    olivedrab: 7048739,
    orange: 16753920,
    orangered: 16729344,
    orchid: 14315734,
    palegoldenrod: 15657130,
    palegreen: 10025880,
    paleturquoise: 11529966,
    palevioletred: 14381203,
    papayawhip: 16773077,
    peachpuff: 16767673,
    peru: 13468991,
    pink: 16761035,
    plum: 14524637,
    powderblue: 11591910,
    purple: 8388736,
    rebeccapurple: 6697881,
    red: 16711680,
    rosybrown: 12357519,
    royalblue: 4286945,
    saddlebrown: 9127187,
    salmon: 16416882,
    sandybrown: 16032864,
    seagreen: 3050327,
    seashell: 16774638,
    sienna: 10506797,
    silver: 12632256,
    skyblue: 8900331,
    slateblue: 6970061,
    slategray: 7372944,
    slategrey: 7372944,
    snow: 16775930,
    springgreen: 65407,
    steelblue: 4620980,
    tan: 13808780,
    teal: 32896,
    thistle: 14204888,
    tomato: 16737095,
    turquoise: 4251856,
    violet: 15631086,
    wheat: 16113331,
    white: 16777215,
    whitesmoke: 16119285,
    yellow: 16776960,
    yellowgreen: 10145074
  };
  define_default(Color, color, {
    copy(channels) {
      return Object.assign(new this.constructor(), this, channels);
    },
    displayable() {
      return this.rgb().displayable();
    },
    hex: color_formatHex,
    // Deprecated! Use color.formatHex.
    formatHex: color_formatHex,
    formatHex8: color_formatHex8,
    formatHsl: color_formatHsl,
    formatRgb: color_formatRgb,
    toString: color_formatRgb
  });
  function color_formatHex() {
    return this.rgb().formatHex();
  }
  function color_formatHex8() {
    return this.rgb().formatHex8();
  }
  function color_formatHsl() {
    return hslConvert(this).formatHsl();
  }
  function color_formatRgb() {
    return this.rgb().formatRgb();
  }
  function color(format) {
    var m2, l;
    format = (format + "").trim().toLowerCase();
    return (m2 = reHex.exec(format)) ? (l = m2[1].length, m2 = parseInt(m2[1], 16), l === 6 ? rgbn(m2) : l === 3 ? new Rgb(m2 >> 8 & 15 | m2 >> 4 & 240, m2 >> 4 & 15 | m2 & 240, (m2 & 15) << 4 | m2 & 15, 1) : l === 8 ? rgba(m2 >> 24 & 255, m2 >> 16 & 255, m2 >> 8 & 255, (m2 & 255) / 255) : l === 4 ? rgba(m2 >> 12 & 15 | m2 >> 8 & 240, m2 >> 8 & 15 | m2 >> 4 & 240, m2 >> 4 & 15 | m2 & 240, ((m2 & 15) << 4 | m2 & 15) / 255) : null) : (m2 = reRgbInteger.exec(format)) ? new Rgb(m2[1], m2[2], m2[3], 1) : (m2 = reRgbPercent.exec(format)) ? new Rgb(m2[1] * 255 / 100, m2[2] * 255 / 100, m2[3] * 255 / 100, 1) : (m2 = reRgbaInteger.exec(format)) ? rgba(m2[1], m2[2], m2[3], m2[4]) : (m2 = reRgbaPercent.exec(format)) ? rgba(m2[1] * 255 / 100, m2[2] * 255 / 100, m2[3] * 255 / 100, m2[4]) : (m2 = reHslPercent.exec(format)) ? hsla(m2[1], m2[2] / 100, m2[3] / 100, 1) : (m2 = reHslaPercent.exec(format)) ? hsla(m2[1], m2[2] / 100, m2[3] / 100, m2[4]) : named.hasOwnProperty(format) ? rgbn(named[format]) : format === "transparent" ? new Rgb(NaN, NaN, NaN, 0) : null;
  }
  function rgbn(n) {
    return new Rgb(n >> 16 & 255, n >> 8 & 255, n & 255, 1);
  }
  function rgba(r, g, b, a2) {
    if (a2 <= 0) r = g = b = NaN;
    return new Rgb(r, g, b, a2);
  }
  function rgbConvert(o) {
    if (!(o instanceof Color)) o = color(o);
    if (!o) return new Rgb();
    o = o.rgb();
    return new Rgb(o.r, o.g, o.b, o.opacity);
  }
  function rgb(r, g, b, opacity) {
    return arguments.length === 1 ? rgbConvert(r) : new Rgb(r, g, b, opacity == null ? 1 : opacity);
  }
  function Rgb(r, g, b, opacity) {
    this.r = +r;
    this.g = +g;
    this.b = +b;
    this.opacity = +opacity;
  }
  define_default(Rgb, rgb, extend(Color, {
    brighter(k) {
      k = k == null ? brighter : Math.pow(brighter, k);
      return new Rgb(this.r * k, this.g * k, this.b * k, this.opacity);
    },
    darker(k) {
      k = k == null ? darker : Math.pow(darker, k);
      return new Rgb(this.r * k, this.g * k, this.b * k, this.opacity);
    },
    rgb() {
      return this;
    },
    clamp() {
      return new Rgb(clampi(this.r), clampi(this.g), clampi(this.b), clampa(this.opacity));
    },
    displayable() {
      return -0.5 <= this.r && this.r < 255.5 && (-0.5 <= this.g && this.g < 255.5) && (-0.5 <= this.b && this.b < 255.5) && (0 <= this.opacity && this.opacity <= 1);
    },
    hex: rgb_formatHex,
    // Deprecated! Use color.formatHex.
    formatHex: rgb_formatHex,
    formatHex8: rgb_formatHex8,
    formatRgb: rgb_formatRgb,
    toString: rgb_formatRgb
  }));
  function rgb_formatHex() {
    return `#${hex(this.r)}${hex(this.g)}${hex(this.b)}`;
  }
  function rgb_formatHex8() {
    return `#${hex(this.r)}${hex(this.g)}${hex(this.b)}${hex((isNaN(this.opacity) ? 1 : this.opacity) * 255)}`;
  }
  function rgb_formatRgb() {
    const a2 = clampa(this.opacity);
    return `${a2 === 1 ? "rgb(" : "rgba("}${clampi(this.r)}, ${clampi(this.g)}, ${clampi(this.b)}${a2 === 1 ? ")" : `, ${a2})`}`;
  }
  function clampa(opacity) {
    return isNaN(opacity) ? 1 : Math.max(0, Math.min(1, opacity));
  }
  function clampi(value) {
    return Math.max(0, Math.min(255, Math.round(value) || 0));
  }
  function hex(value) {
    value = clampi(value);
    return (value < 16 ? "0" : "") + value.toString(16);
  }
  function hsla(h, s, l, a2) {
    if (a2 <= 0) h = s = l = NaN;
    else if (l <= 0 || l >= 1) h = s = NaN;
    else if (s <= 0) h = NaN;
    return new Hsl(h, s, l, a2);
  }
  function hslConvert(o) {
    if (o instanceof Hsl) return new Hsl(o.h, o.s, o.l, o.opacity);
    if (!(o instanceof Color)) o = color(o);
    if (!o) return new Hsl();
    if (o instanceof Hsl) return o;
    o = o.rgb();
    var r = o.r / 255, g = o.g / 255, b = o.b / 255, min2 = Math.min(r, g, b), max2 = Math.max(r, g, b), h = NaN, s = max2 - min2, l = (max2 + min2) / 2;
    if (s) {
      if (r === max2) h = (g - b) / s + (g < b) * 6;
      else if (g === max2) h = (b - r) / s + 2;
      else h = (r - g) / s + 4;
      s /= l < 0.5 ? max2 + min2 : 2 - max2 - min2;
      h *= 60;
    } else {
      s = l > 0 && l < 1 ? 0 : h;
    }
    return new Hsl(h, s, l, o.opacity);
  }
  function hsl(h, s, l, opacity) {
    return arguments.length === 1 ? hslConvert(h) : new Hsl(h, s, l, opacity == null ? 1 : opacity);
  }
  function Hsl(h, s, l, opacity) {
    this.h = +h;
    this.s = +s;
    this.l = +l;
    this.opacity = +opacity;
  }
  define_default(Hsl, hsl, extend(Color, {
    brighter(k) {
      k = k == null ? brighter : Math.pow(brighter, k);
      return new Hsl(this.h, this.s, this.l * k, this.opacity);
    },
    darker(k) {
      k = k == null ? darker : Math.pow(darker, k);
      return new Hsl(this.h, this.s, this.l * k, this.opacity);
    },
    rgb() {
      var h = this.h % 360 + (this.h < 0) * 360, s = isNaN(h) || isNaN(this.s) ? 0 : this.s, l = this.l, m2 = l + (l < 0.5 ? l : 1 - l) * s, m1 = 2 * l - m2;
      return new Rgb(
        hsl2rgb(h >= 240 ? h - 240 : h + 120, m1, m2),
        hsl2rgb(h, m1, m2),
        hsl2rgb(h < 120 ? h + 240 : h - 120, m1, m2),
        this.opacity
      );
    },
    clamp() {
      return new Hsl(clamph(this.h), clampt(this.s), clampt(this.l), clampa(this.opacity));
    },
    displayable() {
      return (0 <= this.s && this.s <= 1 || isNaN(this.s)) && (0 <= this.l && this.l <= 1) && (0 <= this.opacity && this.opacity <= 1);
    },
    formatHsl() {
      const a2 = clampa(this.opacity);
      return `${a2 === 1 ? "hsl(" : "hsla("}${clamph(this.h)}, ${clampt(this.s) * 100}%, ${clampt(this.l) * 100}%${a2 === 1 ? ")" : `, ${a2})`}`;
    }
  }));
  function clamph(value) {
    value = (value || 0) % 360;
    return value < 0 ? value + 360 : value;
  }
  function clampt(value) {
    return Math.max(0, Math.min(1, value || 0));
  }
  function hsl2rgb(h, m1, m2) {
    return (h < 60 ? m1 + (m2 - m1) * h / 60 : h < 180 ? m2 : h < 240 ? m1 + (m2 - m1) * (240 - h) / 60 : m1) * 255;
  }

  // node_modules/d3-interpolate/src/basis.js
  function basis(t1, v0, v1, v2, v3) {
    var t2 = t1 * t1, t3 = t2 * t1;
    return ((1 - 3 * t1 + 3 * t2 - t3) * v0 + (4 - 6 * t2 + 3 * t3) * v1 + (1 + 3 * t1 + 3 * t2 - 3 * t3) * v2 + t3 * v3) / 6;
  }
  function basis_default(values) {
    var n = values.length - 1;
    return function(t) {
      var i = t <= 0 ? t = 0 : t >= 1 ? (t = 1, n - 1) : Math.floor(t * n), v1 = values[i], v2 = values[i + 1], v0 = i > 0 ? values[i - 1] : 2 * v1 - v2, v3 = i < n - 1 ? values[i + 2] : 2 * v2 - v1;
      return basis((t - i / n) * n, v0, v1, v2, v3);
    };
  }

  // node_modules/d3-interpolate/src/basisClosed.js
  function basisClosed_default(values) {
    var n = values.length;
    return function(t) {
      var i = Math.floor(((t %= 1) < 0 ? ++t : t) * n), v0 = values[(i + n - 1) % n], v1 = values[i % n], v2 = values[(i + 1) % n], v3 = values[(i + 2) % n];
      return basis((t - i / n) * n, v0, v1, v2, v3);
    };
  }

  // node_modules/d3-interpolate/src/constant.js
  var constant_default2 = (x3) => () => x3;

  // node_modules/d3-interpolate/src/color.js
  function linear(a2, d) {
    return function(t) {
      return a2 + t * d;
    };
  }
  function exponential(a2, b, y3) {
    return a2 = Math.pow(a2, y3), b = Math.pow(b, y3) - a2, y3 = 1 / y3, function(t) {
      return Math.pow(a2 + t * b, y3);
    };
  }
  function gamma(y3) {
    return (y3 = +y3) === 1 ? nogamma : function(a2, b) {
      return b - a2 ? exponential(a2, b, y3) : constant_default2(isNaN(a2) ? b : a2);
    };
  }
  function nogamma(a2, b) {
    var d = b - a2;
    return d ? linear(a2, d) : constant_default2(isNaN(a2) ? b : a2);
  }

  // node_modules/d3-interpolate/src/rgb.js
  var rgb_default = function rgbGamma(y3) {
    var color2 = gamma(y3);
    function rgb2(start2, end) {
      var r = color2((start2 = rgb(start2)).r, (end = rgb(end)).r), g = color2(start2.g, end.g), b = color2(start2.b, end.b), opacity = nogamma(start2.opacity, end.opacity);
      return function(t) {
        start2.r = r(t);
        start2.g = g(t);
        start2.b = b(t);
        start2.opacity = opacity(t);
        return start2 + "";
      };
    }
    rgb2.gamma = rgbGamma;
    return rgb2;
  }(1);
  function rgbSpline(spline) {
    return function(colors) {
      var n = colors.length, r = new Array(n), g = new Array(n), b = new Array(n), i, color2;
      for (i = 0; i < n; ++i) {
        color2 = rgb(colors[i]);
        r[i] = color2.r || 0;
        g[i] = color2.g || 0;
        b[i] = color2.b || 0;
      }
      r = spline(r);
      g = spline(g);
      b = spline(b);
      color2.opacity = 1;
      return function(t) {
        color2.r = r(t);
        color2.g = g(t);
        color2.b = b(t);
        return color2 + "";
      };
    };
  }
  var rgbBasis = rgbSpline(basis_default);
  var rgbBasisClosed = rgbSpline(basisClosed_default);

  // node_modules/d3-interpolate/src/number.js
  function number_default(a2, b) {
    return a2 = +a2, b = +b, function(t) {
      return a2 * (1 - t) + b * t;
    };
  }

  // node_modules/d3-interpolate/src/string.js
  var reA = /[-+]?(?:\d+\.?\d*|\.?\d+)(?:[eE][-+]?\d+)?/g;
  var reB = new RegExp(reA.source, "g");
  function zero(b) {
    return function() {
      return b;
    };
  }
  function one(b) {
    return function(t) {
      return b(t) + "";
    };
  }
  function string_default(a2, b) {
    var bi = reA.lastIndex = reB.lastIndex = 0, am, bm, bs, i = -1, s = [], q = [];
    a2 = a2 + "", b = b + "";
    while ((am = reA.exec(a2)) && (bm = reB.exec(b))) {
      if ((bs = bm.index) > bi) {
        bs = b.slice(bi, bs);
        if (s[i]) s[i] += bs;
        else s[++i] = bs;
      }
      if ((am = am[0]) === (bm = bm[0])) {
        if (s[i]) s[i] += bm;
        else s[++i] = bm;
      } else {
        s[++i] = null;
        q.push({ i, x: number_default(am, bm) });
      }
      bi = reB.lastIndex;
    }
    if (bi < b.length) {
      bs = b.slice(bi);
      if (s[i]) s[i] += bs;
      else s[++i] = bs;
    }
    return s.length < 2 ? q[0] ? one(q[0].x) : zero(b) : (b = q.length, function(t) {
      for (var i2 = 0, o; i2 < b; ++i2) s[(o = q[i2]).i] = o.x(t);
      return s.join("");
    });
  }

  // node_modules/d3-interpolate/src/transform/decompose.js
  var degrees = 180 / Math.PI;
  var identity = {
    translateX: 0,
    translateY: 0,
    rotate: 0,
    skewX: 0,
    scaleX: 1,
    scaleY: 1
  };
  function decompose_default(a2, b, c2, d, e, f) {
    var scaleX, scaleY, skewX;
    if (scaleX = Math.sqrt(a2 * a2 + b * b)) a2 /= scaleX, b /= scaleX;
    if (skewX = a2 * c2 + b * d) c2 -= a2 * skewX, d -= b * skewX;
    if (scaleY = Math.sqrt(c2 * c2 + d * d)) c2 /= scaleY, d /= scaleY, skewX /= scaleY;
    if (a2 * d < b * c2) a2 = -a2, b = -b, skewX = -skewX, scaleX = -scaleX;
    return {
      translateX: e,
      translateY: f,
      rotate: Math.atan2(b, a2) * degrees,
      skewX: Math.atan(skewX) * degrees,
      scaleX,
      scaleY
    };
  }

  // node_modules/d3-interpolate/src/transform/parse.js
  var svgNode;
  function parseCss(value) {
    const m2 = new (typeof DOMMatrix === "function" ? DOMMatrix : WebKitCSSMatrix)(value + "");
    return m2.isIdentity ? identity : decompose_default(m2.a, m2.b, m2.c, m2.d, m2.e, m2.f);
  }
  function parseSvg(value) {
    if (value == null) return identity;
    if (!svgNode) svgNode = document.createElementNS("http://www.w3.org/2000/svg", "g");
    svgNode.setAttribute("transform", value);
    if (!(value = svgNode.transform.baseVal.consolidate())) return identity;
    value = value.matrix;
    return decompose_default(value.a, value.b, value.c, value.d, value.e, value.f);
  }

  // node_modules/d3-interpolate/src/transform/index.js
  function interpolateTransform(parse, pxComma, pxParen, degParen) {
    function pop(s) {
      return s.length ? s.pop() + " " : "";
    }
    function translate(xa, ya, xb, yb, s, q) {
      if (xa !== xb || ya !== yb) {
        var i = s.push("translate(", null, pxComma, null, pxParen);
        q.push({ i: i - 4, x: number_default(xa, xb) }, { i: i - 2, x: number_default(ya, yb) });
      } else if (xb || yb) {
        s.push("translate(" + xb + pxComma + yb + pxParen);
      }
    }
    function rotate(a2, b, s, q) {
      if (a2 !== b) {
        if (a2 - b > 180) b += 360;
        else if (b - a2 > 180) a2 += 360;
        q.push({ i: s.push(pop(s) + "rotate(", null, degParen) - 2, x: number_default(a2, b) });
      } else if (b) {
        s.push(pop(s) + "rotate(" + b + degParen);
      }
    }
    function skewX(a2, b, s, q) {
      if (a2 !== b) {
        q.push({ i: s.push(pop(s) + "skewX(", null, degParen) - 2, x: number_default(a2, b) });
      } else if (b) {
        s.push(pop(s) + "skewX(" + b + degParen);
      }
    }
    function scale(xa, ya, xb, yb, s, q) {
      if (xa !== xb || ya !== yb) {
        var i = s.push(pop(s) + "scale(", null, ",", null, ")");
        q.push({ i: i - 4, x: number_default(xa, xb) }, { i: i - 2, x: number_default(ya, yb) });
      } else if (xb !== 1 || yb !== 1) {
        s.push(pop(s) + "scale(" + xb + "," + yb + ")");
      }
    }
    return function(a2, b) {
      var s = [], q = [];
      a2 = parse(a2), b = parse(b);
      translate(a2.translateX, a2.translateY, b.translateX, b.translateY, s, q);
      rotate(a2.rotate, b.rotate, s, q);
      skewX(a2.skewX, b.skewX, s, q);
      scale(a2.scaleX, a2.scaleY, b.scaleX, b.scaleY, s, q);
      a2 = b = null;
      return function(t) {
        var i = -1, n = q.length, o;
        while (++i < n) s[(o = q[i]).i] = o.x(t);
        return s.join("");
      };
    };
  }
  var interpolateTransformCss = interpolateTransform(parseCss, "px, ", "px)", "deg)");
  var interpolateTransformSvg = interpolateTransform(parseSvg, ", ", ")", ")");

  // node_modules/d3-timer/src/timer.js
  var frame = 0;
  var timeout = 0;
  var interval = 0;
  var pokeDelay = 1e3;
  var taskHead;
  var taskTail;
  var clockLast = 0;
  var clockNow = 0;
  var clockSkew = 0;
  var clock = typeof performance === "object" && performance.now ? performance : Date;
  var setFrame = typeof window === "object" && window.requestAnimationFrame ? window.requestAnimationFrame.bind(window) : function(f) {
    setTimeout(f, 17);
  };
  function now() {
    return clockNow || (setFrame(clearNow), clockNow = clock.now() + clockSkew);
  }
  function clearNow() {
    clockNow = 0;
  }
  function Timer() {
    this._call = this._time = this._next = null;
  }
  Timer.prototype = timer.prototype = {
    constructor: Timer,
    restart: function(callback, delay, time) {
      if (typeof callback !== "function") throw new TypeError("callback is not a function");
      time = (time == null ? now() : +time) + (delay == null ? 0 : +delay);
      if (!this._next && taskTail !== this) {
        if (taskTail) taskTail._next = this;
        else taskHead = this;
        taskTail = this;
      }
      this._call = callback;
      this._time = time;
      sleep();
    },
    stop: function() {
      if (this._call) {
        this._call = null;
        this._time = Infinity;
        sleep();
      }
    }
  };
  function timer(callback, delay, time) {
    var t = new Timer();
    t.restart(callback, delay, time);
    return t;
  }
  function timerFlush() {
    now();
    ++frame;
    var t = taskHead, e;
    while (t) {
      if ((e = clockNow - t._time) >= 0) t._call.call(void 0, e);
      t = t._next;
    }
    --frame;
  }
  function wake() {
    clockNow = (clockLast = clock.now()) + clockSkew;
    frame = timeout = 0;
    try {
      timerFlush();
    } finally {
      frame = 0;
      nap();
      clockNow = 0;
    }
  }
  function poke() {
    var now2 = clock.now(), delay = now2 - clockLast;
    if (delay > pokeDelay) clockSkew -= delay, clockLast = now2;
  }
  function nap() {
    var t0, t1 = taskHead, t2, time = Infinity;
    while (t1) {
      if (t1._call) {
        if (time > t1._time) time = t1._time;
        t0 = t1, t1 = t1._next;
      } else {
        t2 = t1._next, t1._next = null;
        t1 = t0 ? t0._next = t2 : taskHead = t2;
      }
    }
    taskTail = t0;
    sleep(time);
  }
  function sleep(time) {
    if (frame) return;
    if (timeout) timeout = clearTimeout(timeout);
    var delay = time - clockNow;
    if (delay > 24) {
      if (time < Infinity) timeout = setTimeout(wake, time - clock.now() - clockSkew);
      if (interval) interval = clearInterval(interval);
    } else {
      if (!interval) clockLast = clock.now(), interval = setInterval(poke, pokeDelay);
      frame = 1, setFrame(wake);
    }
  }

  // node_modules/d3-timer/src/timeout.js
  function timeout_default(callback, delay, time) {
    var t = new Timer();
    delay = delay == null ? 0 : +delay;
    t.restart((elapsed) => {
      t.stop();
      callback(elapsed + delay);
    }, delay, time);
    return t;
  }

  // node_modules/d3-transition/src/transition/schedule.js
  var emptyOn = dispatch_default("start", "end", "cancel", "interrupt");
  var emptyTween = [];
  var CREATED = 0;
  var SCHEDULED = 1;
  var STARTING = 2;
  var STARTED = 3;
  var RUNNING = 4;
  var ENDING = 5;
  var ENDED = 6;
  function schedule_default(node, name, id2, index2, group, timing) {
    var schedules = node.__transition;
    if (!schedules) node.__transition = {};
    else if (id2 in schedules) return;
    create(node, id2, {
      name,
      index: index2,
      // For context during callback.
      group,
      // For context during callback.
      on: emptyOn,
      tween: emptyTween,
      time: timing.time,
      delay: timing.delay,
      duration: timing.duration,
      ease: timing.ease,
      timer: null,
      state: CREATED
    });
  }
  function init(node, id2) {
    var schedule = get2(node, id2);
    if (schedule.state > CREATED) throw new Error("too late; already scheduled");
    return schedule;
  }
  function set2(node, id2) {
    var schedule = get2(node, id2);
    if (schedule.state > STARTED) throw new Error("too late; already running");
    return schedule;
  }
  function get2(node, id2) {
    var schedule = node.__transition;
    if (!schedule || !(schedule = schedule[id2])) throw new Error("transition not found");
    return schedule;
  }
  function create(node, id2, self2) {
    var schedules = node.__transition, tween;
    schedules[id2] = self2;
    self2.timer = timer(schedule, 0, self2.time);
    function schedule(elapsed) {
      self2.state = SCHEDULED;
      self2.timer.restart(start2, self2.delay, self2.time);
      if (self2.delay <= elapsed) start2(elapsed - self2.delay);
    }
    function start2(elapsed) {
      var i, j, n, o;
      if (self2.state !== SCHEDULED) return stop();
      for (i in schedules) {
        o = schedules[i];
        if (o.name !== self2.name) continue;
        if (o.state === STARTED) return timeout_default(start2);
        if (o.state === RUNNING) {
          o.state = ENDED;
          o.timer.stop();
          o.on.call("interrupt", node, node.__data__, o.index, o.group);
          delete schedules[i];
        } else if (+i < id2) {
          o.state = ENDED;
          o.timer.stop();
          o.on.call("cancel", node, node.__data__, o.index, o.group);
          delete schedules[i];
        }
      }
      timeout_default(function() {
        if (self2.state === STARTED) {
          self2.state = RUNNING;
          self2.timer.restart(tick, self2.delay, self2.time);
          tick(elapsed);
        }
      });
      self2.state = STARTING;
      self2.on.call("start", node, node.__data__, self2.index, self2.group);
      if (self2.state !== STARTING) return;
      self2.state = STARTED;
      tween = new Array(n = self2.tween.length);
      for (i = 0, j = -1; i < n; ++i) {
        if (o = self2.tween[i].value.call(node, node.__data__, self2.index, self2.group)) {
          tween[++j] = o;
        }
      }
      tween.length = j + 1;
    }
    function tick(elapsed) {
      var t = elapsed < self2.duration ? self2.ease.call(null, elapsed / self2.duration) : (self2.timer.restart(stop), self2.state = ENDING, 1), i = -1, n = tween.length;
      while (++i < n) {
        tween[i].call(node, t);
      }
      if (self2.state === ENDING) {
        self2.on.call("end", node, node.__data__, self2.index, self2.group);
        stop();
      }
    }
    function stop() {
      self2.state = ENDED;
      self2.timer.stop();
      delete schedules[id2];
      for (var i in schedules) return;
      delete node.__transition;
    }
  }

  // node_modules/d3-transition/src/interrupt.js
  function interrupt_default(node, name) {
    var schedules = node.__transition, schedule, active, empty2 = true, i;
    if (!schedules) return;
    name = name == null ? null : name + "";
    for (i in schedules) {
      if ((schedule = schedules[i]).name !== name) {
        empty2 = false;
        continue;
      }
      active = schedule.state > STARTING && schedule.state < ENDING;
      schedule.state = ENDED;
      schedule.timer.stop();
      schedule.on.call(active ? "interrupt" : "cancel", node, node.__data__, schedule.index, schedule.group);
      delete schedules[i];
    }
    if (empty2) delete node.__transition;
  }

  // node_modules/d3-transition/src/selection/interrupt.js
  function interrupt_default2(name) {
    return this.each(function() {
      interrupt_default(this, name);
    });
  }

  // node_modules/d3-transition/src/transition/tween.js
  function tweenRemove(id2, name) {
    var tween0, tween1;
    return function() {
      var schedule = set2(this, id2), tween = schedule.tween;
      if (tween !== tween0) {
        tween1 = tween0 = tween;
        for (var i = 0, n = tween1.length; i < n; ++i) {
          if (tween1[i].name === name) {
            tween1 = tween1.slice();
            tween1.splice(i, 1);
            break;
          }
        }
      }
      schedule.tween = tween1;
    };
  }
  function tweenFunction(id2, name, value) {
    var tween0, tween1;
    if (typeof value !== "function") throw new Error();
    return function() {
      var schedule = set2(this, id2), tween = schedule.tween;
      if (tween !== tween0) {
        tween1 = (tween0 = tween).slice();
        for (var t = { name, value }, i = 0, n = tween1.length; i < n; ++i) {
          if (tween1[i].name === name) {
            tween1[i] = t;
            break;
          }
        }
        if (i === n) tween1.push(t);
      }
      schedule.tween = tween1;
    };
  }
  function tween_default(name, value) {
    var id2 = this._id;
    name += "";
    if (arguments.length < 2) {
      var tween = get2(this.node(), id2).tween;
      for (var i = 0, n = tween.length, t; i < n; ++i) {
        if ((t = tween[i]).name === name) {
          return t.value;
        }
      }
      return null;
    }
    return this.each((value == null ? tweenRemove : tweenFunction)(id2, name, value));
  }
  function tweenValue(transition2, name, value) {
    var id2 = transition2._id;
    transition2.each(function() {
      var schedule = set2(this, id2);
      (schedule.value || (schedule.value = {}))[name] = value.apply(this, arguments);
    });
    return function(node) {
      return get2(node, id2).value[name];
    };
  }

  // node_modules/d3-transition/src/transition/interpolate.js
  function interpolate_default(a2, b) {
    var c2;
    return (typeof b === "number" ? number_default : b instanceof color ? rgb_default : (c2 = color(b)) ? (b = c2, rgb_default) : string_default)(a2, b);
  }

  // node_modules/d3-transition/src/transition/attr.js
  function attrRemove2(name) {
    return function() {
      this.removeAttribute(name);
    };
  }
  function attrRemoveNS2(fullname) {
    return function() {
      this.removeAttributeNS(fullname.space, fullname.local);
    };
  }
  function attrConstant2(name, interpolate, value1) {
    var string00, string1 = value1 + "", interpolate0;
    return function() {
      var string0 = this.getAttribute(name);
      return string0 === string1 ? null : string0 === string00 ? interpolate0 : interpolate0 = interpolate(string00 = string0, value1);
    };
  }
  function attrConstantNS2(fullname, interpolate, value1) {
    var string00, string1 = value1 + "", interpolate0;
    return function() {
      var string0 = this.getAttributeNS(fullname.space, fullname.local);
      return string0 === string1 ? null : string0 === string00 ? interpolate0 : interpolate0 = interpolate(string00 = string0, value1);
    };
  }
  function attrFunction2(name, interpolate, value) {
    var string00, string10, interpolate0;
    return function() {
      var string0, value1 = value(this), string1;
      if (value1 == null) return void this.removeAttribute(name);
      string0 = this.getAttribute(name);
      string1 = value1 + "";
      return string0 === string1 ? null : string0 === string00 && string1 === string10 ? interpolate0 : (string10 = string1, interpolate0 = interpolate(string00 = string0, value1));
    };
  }
  function attrFunctionNS2(fullname, interpolate, value) {
    var string00, string10, interpolate0;
    return function() {
      var string0, value1 = value(this), string1;
      if (value1 == null) return void this.removeAttributeNS(fullname.space, fullname.local);
      string0 = this.getAttributeNS(fullname.space, fullname.local);
      string1 = value1 + "";
      return string0 === string1 ? null : string0 === string00 && string1 === string10 ? interpolate0 : (string10 = string1, interpolate0 = interpolate(string00 = string0, value1));
    };
  }
  function attr_default2(name, value) {
    var fullname = namespace_default(name), i = fullname === "transform" ? interpolateTransformSvg : interpolate_default;
    return this.attrTween(name, typeof value === "function" ? (fullname.local ? attrFunctionNS2 : attrFunction2)(fullname, i, tweenValue(this, "attr." + name, value)) : value == null ? (fullname.local ? attrRemoveNS2 : attrRemove2)(fullname) : (fullname.local ? attrConstantNS2 : attrConstant2)(fullname, i, value));
  }

  // node_modules/d3-transition/src/transition/attrTween.js
  function attrInterpolate(name, i) {
    return function(t) {
      this.setAttribute(name, i.call(this, t));
    };
  }
  function attrInterpolateNS(fullname, i) {
    return function(t) {
      this.setAttributeNS(fullname.space, fullname.local, i.call(this, t));
    };
  }
  function attrTweenNS(fullname, value) {
    var t0, i0;
    function tween() {
      var i = value.apply(this, arguments);
      if (i !== i0) t0 = (i0 = i) && attrInterpolateNS(fullname, i);
      return t0;
    }
    tween._value = value;
    return tween;
  }
  function attrTween(name, value) {
    var t0, i0;
    function tween() {
      var i = value.apply(this, arguments);
      if (i !== i0) t0 = (i0 = i) && attrInterpolate(name, i);
      return t0;
    }
    tween._value = value;
    return tween;
  }
  function attrTween_default(name, value) {
    var key = "attr." + name;
    if (arguments.length < 2) return (key = this.tween(key)) && key._value;
    if (value == null) return this.tween(key, null);
    if (typeof value !== "function") throw new Error();
    var fullname = namespace_default(name);
    return this.tween(key, (fullname.local ? attrTweenNS : attrTween)(fullname, value));
  }

  // node_modules/d3-transition/src/transition/delay.js
  function delayFunction(id2, value) {
    return function() {
      init(this, id2).delay = +value.apply(this, arguments);
    };
  }
  function delayConstant(id2, value) {
    return value = +value, function() {
      init(this, id2).delay = value;
    };
  }
  function delay_default(value) {
    var id2 = this._id;
    return arguments.length ? this.each((typeof value === "function" ? delayFunction : delayConstant)(id2, value)) : get2(this.node(), id2).delay;
  }

  // node_modules/d3-transition/src/transition/duration.js
  function durationFunction(id2, value) {
    return function() {
      set2(this, id2).duration = +value.apply(this, arguments);
    };
  }
  function durationConstant(id2, value) {
    return value = +value, function() {
      set2(this, id2).duration = value;
    };
  }
  function duration_default(value) {
    var id2 = this._id;
    return arguments.length ? this.each((typeof value === "function" ? durationFunction : durationConstant)(id2, value)) : get2(this.node(), id2).duration;
  }

  // node_modules/d3-transition/src/transition/ease.js
  function easeConstant(id2, value) {
    if (typeof value !== "function") throw new Error();
    return function() {
      set2(this, id2).ease = value;
    };
  }
  function ease_default(value) {
    var id2 = this._id;
    return arguments.length ? this.each(easeConstant(id2, value)) : get2(this.node(), id2).ease;
  }

  // node_modules/d3-transition/src/transition/easeVarying.js
  function easeVarying(id2, value) {
    return function() {
      var v = value.apply(this, arguments);
      if (typeof v !== "function") throw new Error();
      set2(this, id2).ease = v;
    };
  }
  function easeVarying_default(value) {
    if (typeof value !== "function") throw new Error();
    return this.each(easeVarying(this._id, value));
  }

  // node_modules/d3-transition/src/transition/filter.js
  function filter_default2(match) {
    if (typeof match !== "function") match = matcher_default(match);
    for (var groups = this._groups, m2 = groups.length, subgroups = new Array(m2), j = 0; j < m2; ++j) {
      for (var group = groups[j], n = group.length, subgroup = subgroups[j] = [], node, i = 0; i < n; ++i) {
        if ((node = group[i]) && match.call(node, node.__data__, i, group)) {
          subgroup.push(node);
        }
      }
    }
    return new Transition(subgroups, this._parents, this._name, this._id);
  }

  // node_modules/d3-transition/src/transition/merge.js
  function merge_default2(transition2) {
    if (transition2._id !== this._id) throw new Error();
    for (var groups0 = this._groups, groups1 = transition2._groups, m0 = groups0.length, m1 = groups1.length, m2 = Math.min(m0, m1), merges = new Array(m0), j = 0; j < m2; ++j) {
      for (var group0 = groups0[j], group1 = groups1[j], n = group0.length, merge = merges[j] = new Array(n), node, i = 0; i < n; ++i) {
        if (node = group0[i] || group1[i]) {
          merge[i] = node;
        }
      }
    }
    for (; j < m0; ++j) {
      merges[j] = groups0[j];
    }
    return new Transition(merges, this._parents, this._name, this._id);
  }

  // node_modules/d3-transition/src/transition/on.js
  function start(name) {
    return (name + "").trim().split(/^|\s+/).every(function(t) {
      var i = t.indexOf(".");
      if (i >= 0) t = t.slice(0, i);
      return !t || t === "start";
    });
  }
  function onFunction(id2, name, listener) {
    var on0, on1, sit = start(name) ? init : set2;
    return function() {
      var schedule = sit(this, id2), on = schedule.on;
      if (on !== on0) (on1 = (on0 = on).copy()).on(name, listener);
      schedule.on = on1;
    };
  }
  function on_default2(name, listener) {
    var id2 = this._id;
    return arguments.length < 2 ? get2(this.node(), id2).on.on(name) : this.each(onFunction(id2, name, listener));
  }

  // node_modules/d3-transition/src/transition/remove.js
  function removeFunction(id2) {
    return function() {
      var parent = this.parentNode;
      for (var i in this.__transition) if (+i !== id2) return;
      if (parent) parent.removeChild(this);
    };
  }
  function remove_default2() {
    return this.on("end.remove", removeFunction(this._id));
  }

  // node_modules/d3-transition/src/transition/select.js
  function select_default2(select) {
    var name = this._name, id2 = this._id;
    if (typeof select !== "function") select = selector_default(select);
    for (var groups = this._groups, m2 = groups.length, subgroups = new Array(m2), j = 0; j < m2; ++j) {
      for (var group = groups[j], n = group.length, subgroup = subgroups[j] = new Array(n), node, subnode, i = 0; i < n; ++i) {
        if ((node = group[i]) && (subnode = select.call(node, node.__data__, i, group))) {
          if ("__data__" in node) subnode.__data__ = node.__data__;
          subgroup[i] = subnode;
          schedule_default(subgroup[i], name, id2, i, subgroup, get2(node, id2));
        }
      }
    }
    return new Transition(subgroups, this._parents, name, id2);
  }

  // node_modules/d3-transition/src/transition/selectAll.js
  function selectAll_default2(select) {
    var name = this._name, id2 = this._id;
    if (typeof select !== "function") select = selectorAll_default(select);
    for (var groups = this._groups, m2 = groups.length, subgroups = [], parents = [], j = 0; j < m2; ++j) {
      for (var group = groups[j], n = group.length, node, i = 0; i < n; ++i) {
        if (node = group[i]) {
          for (var children2 = select.call(node, node.__data__, i, group), child, inherit2 = get2(node, id2), k = 0, l = children2.length; k < l; ++k) {
            if (child = children2[k]) {
              schedule_default(child, name, id2, k, children2, inherit2);
            }
          }
          subgroups.push(children2);
          parents.push(node);
        }
      }
    }
    return new Transition(subgroups, parents, name, id2);
  }

  // node_modules/d3-transition/src/transition/selection.js
  var Selection2 = selection_default.prototype.constructor;
  function selection_default2() {
    return new Selection2(this._groups, this._parents);
  }

  // node_modules/d3-transition/src/transition/style.js
  function styleNull(name, interpolate) {
    var string00, string10, interpolate0;
    return function() {
      var string0 = styleValue(this, name), string1 = (this.style.removeProperty(name), styleValue(this, name));
      return string0 === string1 ? null : string0 === string00 && string1 === string10 ? interpolate0 : interpolate0 = interpolate(string00 = string0, string10 = string1);
    };
  }
  function styleRemove2(name) {
    return function() {
      this.style.removeProperty(name);
    };
  }
  function styleConstant2(name, interpolate, value1) {
    var string00, string1 = value1 + "", interpolate0;
    return function() {
      var string0 = styleValue(this, name);
      return string0 === string1 ? null : string0 === string00 ? interpolate0 : interpolate0 = interpolate(string00 = string0, value1);
    };
  }
  function styleFunction2(name, interpolate, value) {
    var string00, string10, interpolate0;
    return function() {
      var string0 = styleValue(this, name), value1 = value(this), string1 = value1 + "";
      if (value1 == null) string1 = value1 = (this.style.removeProperty(name), styleValue(this, name));
      return string0 === string1 ? null : string0 === string00 && string1 === string10 ? interpolate0 : (string10 = string1, interpolate0 = interpolate(string00 = string0, value1));
    };
  }
  function styleMaybeRemove(id2, name) {
    var on0, on1, listener0, key = "style." + name, event = "end." + key, remove2;
    return function() {
      var schedule = set2(this, id2), on = schedule.on, listener = schedule.value[key] == null ? remove2 || (remove2 = styleRemove2(name)) : void 0;
      if (on !== on0 || listener0 !== listener) (on1 = (on0 = on).copy()).on(event, listener0 = listener);
      schedule.on = on1;
    };
  }
  function style_default2(name, value, priority) {
    var i = (name += "") === "transform" ? interpolateTransformCss : interpolate_default;
    return value == null ? this.styleTween(name, styleNull(name, i)).on("end.style." + name, styleRemove2(name)) : typeof value === "function" ? this.styleTween(name, styleFunction2(name, i, tweenValue(this, "style." + name, value))).each(styleMaybeRemove(this._id, name)) : this.styleTween(name, styleConstant2(name, i, value), priority).on("end.style." + name, null);
  }

  // node_modules/d3-transition/src/transition/styleTween.js
  function styleInterpolate(name, i, priority) {
    return function(t) {
      this.style.setProperty(name, i.call(this, t), priority);
    };
  }
  function styleTween(name, value, priority) {
    var t, i0;
    function tween() {
      var i = value.apply(this, arguments);
      if (i !== i0) t = (i0 = i) && styleInterpolate(name, i, priority);
      return t;
    }
    tween._value = value;
    return tween;
  }
  function styleTween_default(name, value, priority) {
    var key = "style." + (name += "");
    if (arguments.length < 2) return (key = this.tween(key)) && key._value;
    if (value == null) return this.tween(key, null);
    if (typeof value !== "function") throw new Error();
    return this.tween(key, styleTween(name, value, priority == null ? "" : priority));
  }

  // node_modules/d3-transition/src/transition/text.js
  function textConstant2(value) {
    return function() {
      this.textContent = value;
    };
  }
  function textFunction2(value) {
    return function() {
      var value1 = value(this);
      this.textContent = value1 == null ? "" : value1;
    };
  }
  function text_default2(value) {
    return this.tween("text", typeof value === "function" ? textFunction2(tweenValue(this, "text", value)) : textConstant2(value == null ? "" : value + ""));
  }

  // node_modules/d3-transition/src/transition/textTween.js
  function textInterpolate(i) {
    return function(t) {
      this.textContent = i.call(this, t);
    };
  }
  function textTween(value) {
    var t0, i0;
    function tween() {
      var i = value.apply(this, arguments);
      if (i !== i0) t0 = (i0 = i) && textInterpolate(i);
      return t0;
    }
    tween._value = value;
    return tween;
  }
  function textTween_default(value) {
    var key = "text";
    if (arguments.length < 1) return (key = this.tween(key)) && key._value;
    if (value == null) return this.tween(key, null);
    if (typeof value !== "function") throw new Error();
    return this.tween(key, textTween(value));
  }

  // node_modules/d3-transition/src/transition/transition.js
  function transition_default() {
    var name = this._name, id0 = this._id, id1 = newId();
    for (var groups = this._groups, m2 = groups.length, j = 0; j < m2; ++j) {
      for (var group = groups[j], n = group.length, node, i = 0; i < n; ++i) {
        if (node = group[i]) {
          var inherit2 = get2(node, id0);
          schedule_default(node, name, id1, i, group, {
            time: inherit2.time + inherit2.delay + inherit2.duration,
            delay: 0,
            duration: inherit2.duration,
            ease: inherit2.ease
          });
        }
      }
    }
    return new Transition(groups, this._parents, name, id1);
  }

  // node_modules/d3-transition/src/transition/end.js
  function end_default() {
    var on0, on1, that = this, id2 = that._id, size = that.size();
    return new Promise(function(resolve, reject) {
      var cancel = { value: reject }, end = { value: function() {
        if (--size === 0) resolve();
      } };
      that.each(function() {
        var schedule = set2(this, id2), on = schedule.on;
        if (on !== on0) {
          on1 = (on0 = on).copy();
          on1._.cancel.push(cancel);
          on1._.interrupt.push(cancel);
          on1._.end.push(end);
        }
        schedule.on = on1;
      });
      if (size === 0) resolve();
    });
  }

  // node_modules/d3-transition/src/transition/index.js
  var id = 0;
  function Transition(groups, parents, name, id2) {
    this._groups = groups;
    this._parents = parents;
    this._name = name;
    this._id = id2;
  }
  function transition(name) {
    return selection_default().transition(name);
  }
  function newId() {
    return ++id;
  }
  var selection_prototype = selection_default.prototype;
  Transition.prototype = transition.prototype = {
    constructor: Transition,
    select: select_default2,
    selectAll: selectAll_default2,
    selectChild: selection_prototype.selectChild,
    selectChildren: selection_prototype.selectChildren,
    filter: filter_default2,
    merge: merge_default2,
    selection: selection_default2,
    transition: transition_default,
    call: selection_prototype.call,
    nodes: selection_prototype.nodes,
    node: selection_prototype.node,
    size: selection_prototype.size,
    empty: selection_prototype.empty,
    each: selection_prototype.each,
    on: on_default2,
    attr: attr_default2,
    attrTween: attrTween_default,
    style: style_default2,
    styleTween: styleTween_default,
    text: text_default2,
    textTween: textTween_default,
    remove: remove_default2,
    tween: tween_default,
    delay: delay_default,
    duration: duration_default,
    ease: ease_default,
    easeVarying: easeVarying_default,
    end: end_default,
    [Symbol.iterator]: selection_prototype[Symbol.iterator]
  };

  // node_modules/d3-ease/src/cubic.js
  function cubicInOut(t) {
    return ((t *= 2) <= 1 ? t * t * t : (t -= 2) * t * t + 2) / 2;
  }

  // node_modules/d3-transition/src/selection/transition.js
  var defaultTiming = {
    time: null,
    // Set on use.
    delay: 0,
    duration: 250,
    ease: cubicInOut
  };
  function inherit(node, id2) {
    var timing;
    while (!(timing = node.__transition) || !(timing = timing[id2])) {
      if (!(node = node.parentNode)) {
        throw new Error(`transition ${id2} not found`);
      }
    }
    return timing;
  }
  function transition_default2(name) {
    var id2, timing;
    if (name instanceof Transition) {
      id2 = name._id, name = name._name;
    } else {
      id2 = newId(), (timing = defaultTiming).time = now(), name = name == null ? null : name + "";
    }
    for (var groups = this._groups, m2 = groups.length, j = 0; j < m2; ++j) {
      for (var group = groups[j], n = group.length, node, i = 0; i < n; ++i) {
        if (node = group[i]) {
          schedule_default(node, name, id2, i, group, timing || inherit(node, id2));
        }
      }
    }
    return new Transition(groups, this._parents, name, id2);
  }

  // node_modules/d3-transition/src/selection/index.js
  selection_default.prototype.interrupt = interrupt_default2;
  selection_default.prototype.transition = transition_default2;

  // node_modules/d3-brush/src/brush.js
  var { abs, max, min } = Math;
  function number1(e) {
    return [+e[0], +e[1]];
  }
  function number2(e) {
    return [number1(e[0]), number1(e[1])];
  }
  var X = {
    name: "x",
    handles: ["w", "e"].map(type),
    input: function(x3, e) {
      return x3 == null ? null : [[+x3[0], e[0][1]], [+x3[1], e[1][1]]];
    },
    output: function(xy) {
      return xy && [xy[0][0], xy[1][0]];
    }
  };
  var Y = {
    name: "y",
    handles: ["n", "s"].map(type),
    input: function(y3, e) {
      return y3 == null ? null : [[e[0][0], +y3[0]], [e[1][0], +y3[1]]];
    },
    output: function(xy) {
      return xy && [xy[0][1], xy[1][1]];
    }
  };
  var XY = {
    name: "xy",
    handles: ["n", "w", "e", "s", "nw", "ne", "sw", "se"].map(type),
    input: function(xy) {
      return xy == null ? null : number2(xy);
    },
    output: function(xy) {
      return xy;
    }
  };
  function type(t) {
    return { type: t };
  }

  // node_modules/d3-force/src/center.js
  function center_default(x3, y3) {
    var nodes, strength = 1;
    if (x3 == null) x3 = 0;
    if (y3 == null) y3 = 0;
    function force() {
      var i, n = nodes.length, node, sx = 0, sy = 0;
      for (i = 0; i < n; ++i) {
        node = nodes[i], sx += node.x, sy += node.y;
      }
      for (sx = (sx / n - x3) * strength, sy = (sy / n - y3) * strength, i = 0; i < n; ++i) {
        node = nodes[i], node.x -= sx, node.y -= sy;
      }
    }
    force.initialize = function(_) {
      nodes = _;
    };
    force.x = function(_) {
      return arguments.length ? (x3 = +_, force) : x3;
    };
    force.y = function(_) {
      return arguments.length ? (y3 = +_, force) : y3;
    };
    force.strength = function(_) {
      return arguments.length ? (strength = +_, force) : strength;
    };
    return force;
  }

  // node_modules/d3-quadtree/src/add.js
  function add_default(d) {
    const x3 = +this._x.call(null, d), y3 = +this._y.call(null, d);
    return add(this.cover(x3, y3), x3, y3, d);
  }
  function add(tree, x3, y3, d) {
    if (isNaN(x3) || isNaN(y3)) return tree;
    var parent, node = tree._root, leaf = { data: d }, x0 = tree._x0, y0 = tree._y0, x1 = tree._x1, y1 = tree._y1, xm, ym, xp, yp, right, bottom, i, j;
    if (!node) return tree._root = leaf, tree;
    while (node.length) {
      if (right = x3 >= (xm = (x0 + x1) / 2)) x0 = xm;
      else x1 = xm;
      if (bottom = y3 >= (ym = (y0 + y1) / 2)) y0 = ym;
      else y1 = ym;
      if (parent = node, !(node = node[i = bottom << 1 | right])) return parent[i] = leaf, tree;
    }
    xp = +tree._x.call(null, node.data);
    yp = +tree._y.call(null, node.data);
    if (x3 === xp && y3 === yp) return leaf.next = node, parent ? parent[i] = leaf : tree._root = leaf, tree;
    do {
      parent = parent ? parent[i] = new Array(4) : tree._root = new Array(4);
      if (right = x3 >= (xm = (x0 + x1) / 2)) x0 = xm;
      else x1 = xm;
      if (bottom = y3 >= (ym = (y0 + y1) / 2)) y0 = ym;
      else y1 = ym;
    } while ((i = bottom << 1 | right) === (j = (yp >= ym) << 1 | xp >= xm));
    return parent[j] = node, parent[i] = leaf, tree;
  }
  function addAll(data) {
    var d, i, n = data.length, x3, y3, xz = new Array(n), yz = new Array(n), x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity;
    for (i = 0; i < n; ++i) {
      if (isNaN(x3 = +this._x.call(null, d = data[i])) || isNaN(y3 = +this._y.call(null, d))) continue;
      xz[i] = x3;
      yz[i] = y3;
      if (x3 < x0) x0 = x3;
      if (x3 > x1) x1 = x3;
      if (y3 < y0) y0 = y3;
      if (y3 > y1) y1 = y3;
    }
    if (x0 > x1 || y0 > y1) return this;
    this.cover(x0, y0).cover(x1, y1);
    for (i = 0; i < n; ++i) {
      add(this, xz[i], yz[i], data[i]);
    }
    return this;
  }

  // node_modules/d3-quadtree/src/cover.js
  function cover_default(x3, y3) {
    if (isNaN(x3 = +x3) || isNaN(y3 = +y3)) return this;
    var x0 = this._x0, y0 = this._y0, x1 = this._x1, y1 = this._y1;
    if (isNaN(x0)) {
      x1 = (x0 = Math.floor(x3)) + 1;
      y1 = (y0 = Math.floor(y3)) + 1;
    } else {
      var z = x1 - x0 || 1, node = this._root, parent, i;
      while (x0 > x3 || x3 >= x1 || y0 > y3 || y3 >= y1) {
        i = (y3 < y0) << 1 | x3 < x0;
        parent = new Array(4), parent[i] = node, node = parent, z *= 2;
        switch (i) {
          case 0:
            x1 = x0 + z, y1 = y0 + z;
            break;
          case 1:
            x0 = x1 - z, y1 = y0 + z;
            break;
          case 2:
            x1 = x0 + z, y0 = y1 - z;
            break;
          case 3:
            x0 = x1 - z, y0 = y1 - z;
            break;
        }
      }
      if (this._root && this._root.length) this._root = node;
    }
    this._x0 = x0;
    this._y0 = y0;
    this._x1 = x1;
    this._y1 = y1;
    return this;
  }

  // node_modules/d3-quadtree/src/data.js
  function data_default2() {
    var data = [];
    this.visit(function(node) {
      if (!node.length) do
        data.push(node.data);
      while (node = node.next);
    });
    return data;
  }

  // node_modules/d3-quadtree/src/extent.js
  function extent_default(_) {
    return arguments.length ? this.cover(+_[0][0], +_[0][1]).cover(+_[1][0], +_[1][1]) : isNaN(this._x0) ? void 0 : [[this._x0, this._y0], [this._x1, this._y1]];
  }

  // node_modules/d3-quadtree/src/quad.js
  function quad_default(node, x0, y0, x1, y1) {
    this.node = node;
    this.x0 = x0;
    this.y0 = y0;
    this.x1 = x1;
    this.y1 = y1;
  }

  // node_modules/d3-quadtree/src/find.js
  function find_default(x3, y3, radius) {
    var data, x0 = this._x0, y0 = this._y0, x1, y1, x22, y22, x32 = this._x1, y32 = this._y1, quads = [], node = this._root, q, i;
    if (node) quads.push(new quad_default(node, x0, y0, x32, y32));
    if (radius == null) radius = Infinity;
    else {
      x0 = x3 - radius, y0 = y3 - radius;
      x32 = x3 + radius, y32 = y3 + radius;
      radius *= radius;
    }
    while (q = quads.pop()) {
      if (!(node = q.node) || (x1 = q.x0) > x32 || (y1 = q.y0) > y32 || (x22 = q.x1) < x0 || (y22 = q.y1) < y0) continue;
      if (node.length) {
        var xm = (x1 + x22) / 2, ym = (y1 + y22) / 2;
        quads.push(
          new quad_default(node[3], xm, ym, x22, y22),
          new quad_default(node[2], x1, ym, xm, y22),
          new quad_default(node[1], xm, y1, x22, ym),
          new quad_default(node[0], x1, y1, xm, ym)
        );
        if (i = (y3 >= ym) << 1 | x3 >= xm) {
          q = quads[quads.length - 1];
          quads[quads.length - 1] = quads[quads.length - 1 - i];
          quads[quads.length - 1 - i] = q;
        }
      } else {
        var dx = x3 - +this._x.call(null, node.data), dy = y3 - +this._y.call(null, node.data), d2 = dx * dx + dy * dy;
        if (d2 < radius) {
          var d = Math.sqrt(radius = d2);
          x0 = x3 - d, y0 = y3 - d;
          x32 = x3 + d, y32 = y3 + d;
          data = node.data;
        }
      }
    }
    return data;
  }

  // node_modules/d3-quadtree/src/remove.js
  function remove_default3(d) {
    if (isNaN(x3 = +this._x.call(null, d)) || isNaN(y3 = +this._y.call(null, d))) return this;
    var parent, node = this._root, retainer, previous, next, x0 = this._x0, y0 = this._y0, x1 = this._x1, y1 = this._y1, x3, y3, xm, ym, right, bottom, i, j;
    if (!node) return this;
    if (node.length) while (true) {
      if (right = x3 >= (xm = (x0 + x1) / 2)) x0 = xm;
      else x1 = xm;
      if (bottom = y3 >= (ym = (y0 + y1) / 2)) y0 = ym;
      else y1 = ym;
      if (!(parent = node, node = node[i = bottom << 1 | right])) return this;
      if (!node.length) break;
      if (parent[i + 1 & 3] || parent[i + 2 & 3] || parent[i + 3 & 3]) retainer = parent, j = i;
    }
    while (node.data !== d) if (!(previous = node, node = node.next)) return this;
    if (next = node.next) delete node.next;
    if (previous) return next ? previous.next = next : delete previous.next, this;
    if (!parent) return this._root = next, this;
    next ? parent[i] = next : delete parent[i];
    if ((node = parent[0] || parent[1] || parent[2] || parent[3]) && node === (parent[3] || parent[2] || parent[1] || parent[0]) && !node.length) {
      if (retainer) retainer[j] = node;
      else this._root = node;
    }
    return this;
  }
  function removeAll(data) {
    for (var i = 0, n = data.length; i < n; ++i) this.remove(data[i]);
    return this;
  }

  // node_modules/d3-quadtree/src/root.js
  function root_default() {
    return this._root;
  }

  // node_modules/d3-quadtree/src/size.js
  function size_default2() {
    var size = 0;
    this.visit(function(node) {
      if (!node.length) do
        ++size;
      while (node = node.next);
    });
    return size;
  }

  // node_modules/d3-quadtree/src/visit.js
  function visit_default(callback) {
    var quads = [], q, node = this._root, child, x0, y0, x1, y1;
    if (node) quads.push(new quad_default(node, this._x0, this._y0, this._x1, this._y1));
    while (q = quads.pop()) {
      if (!callback(node = q.node, x0 = q.x0, y0 = q.y0, x1 = q.x1, y1 = q.y1) && node.length) {
        var xm = (x0 + x1) / 2, ym = (y0 + y1) / 2;
        if (child = node[3]) quads.push(new quad_default(child, xm, ym, x1, y1));
        if (child = node[2]) quads.push(new quad_default(child, x0, ym, xm, y1));
        if (child = node[1]) quads.push(new quad_default(child, xm, y0, x1, ym));
        if (child = node[0]) quads.push(new quad_default(child, x0, y0, xm, ym));
      }
    }
    return this;
  }

  // node_modules/d3-quadtree/src/visitAfter.js
  function visitAfter_default(callback) {
    var quads = [], next = [], q;
    if (this._root) quads.push(new quad_default(this._root, this._x0, this._y0, this._x1, this._y1));
    while (q = quads.pop()) {
      var node = q.node;
      if (node.length) {
        var child, x0 = q.x0, y0 = q.y0, x1 = q.x1, y1 = q.y1, xm = (x0 + x1) / 2, ym = (y0 + y1) / 2;
        if (child = node[0]) quads.push(new quad_default(child, x0, y0, xm, ym));
        if (child = node[1]) quads.push(new quad_default(child, xm, y0, x1, ym));
        if (child = node[2]) quads.push(new quad_default(child, x0, ym, xm, y1));
        if (child = node[3]) quads.push(new quad_default(child, xm, ym, x1, y1));
      }
      next.push(q);
    }
    while (q = next.pop()) {
      callback(q.node, q.x0, q.y0, q.x1, q.y1);
    }
    return this;
  }

  // node_modules/d3-quadtree/src/x.js
  function defaultX(d) {
    return d[0];
  }
  function x_default(_) {
    return arguments.length ? (this._x = _, this) : this._x;
  }

  // node_modules/d3-quadtree/src/y.js
  function defaultY(d) {
    return d[1];
  }
  function y_default(_) {
    return arguments.length ? (this._y = _, this) : this._y;
  }

  // node_modules/d3-quadtree/src/quadtree.js
  function quadtree(nodes, x3, y3) {
    var tree = new Quadtree(x3 == null ? defaultX : x3, y3 == null ? defaultY : y3, NaN, NaN, NaN, NaN);
    return nodes == null ? tree : tree.addAll(nodes);
  }
  function Quadtree(x3, y3, x0, y0, x1, y1) {
    this._x = x3;
    this._y = y3;
    this._x0 = x0;
    this._y0 = y0;
    this._x1 = x1;
    this._y1 = y1;
    this._root = void 0;
  }
  function leaf_copy(leaf) {
    var copy = { data: leaf.data }, next = copy;
    while (leaf = leaf.next) next = next.next = { data: leaf.data };
    return copy;
  }
  var treeProto = quadtree.prototype = Quadtree.prototype;
  treeProto.copy = function() {
    var copy = new Quadtree(this._x, this._y, this._x0, this._y0, this._x1, this._y1), node = this._root, nodes, child;
    if (!node) return copy;
    if (!node.length) return copy._root = leaf_copy(node), copy;
    nodes = [{ source: node, target: copy._root = new Array(4) }];
    while (node = nodes.pop()) {
      for (var i = 0; i < 4; ++i) {
        if (child = node.source[i]) {
          if (child.length) nodes.push({ source: child, target: node.target[i] = new Array(4) });
          else node.target[i] = leaf_copy(child);
        }
      }
    }
    return copy;
  };
  treeProto.add = add_default;
  treeProto.addAll = addAll;
  treeProto.cover = cover_default;
  treeProto.data = data_default2;
  treeProto.extent = extent_default;
  treeProto.find = find_default;
  treeProto.remove = remove_default3;
  treeProto.removeAll = removeAll;
  treeProto.root = root_default;
  treeProto.size = size_default2;
  treeProto.visit = visit_default;
  treeProto.visitAfter = visitAfter_default;
  treeProto.x = x_default;
  treeProto.y = y_default;

  // node_modules/d3-force/src/constant.js
  function constant_default4(x3) {
    return function() {
      return x3;
    };
  }

  // node_modules/d3-force/src/jiggle.js
  function jiggle_default(random) {
    return (random() - 0.5) * 1e-6;
  }

  // node_modules/d3-force/src/collide.js
  function x(d) {
    return d.x + d.vx;
  }
  function y(d) {
    return d.y + d.vy;
  }
  function collide_default(radius) {
    var nodes, radii, random, strength = 1, iterations = 1;
    if (typeof radius !== "function") radius = constant_default4(radius == null ? 1 : +radius);
    function force() {
      var i, n = nodes.length, tree, node, xi, yi, ri, ri2;
      for (var k = 0; k < iterations; ++k) {
        tree = quadtree(nodes, x, y).visitAfter(prepare);
        for (i = 0; i < n; ++i) {
          node = nodes[i];
          ri = radii[node.index], ri2 = ri * ri;
          xi = node.x + node.vx;
          yi = node.y + node.vy;
          tree.visit(apply);
        }
      }
      function apply(quad, x0, y0, x1, y1) {
        var data = quad.data, rj = quad.r, r = ri + rj;
        if (data) {
          if (data.index > node.index) {
            var x3 = xi - data.x - data.vx, y3 = yi - data.y - data.vy, l = x3 * x3 + y3 * y3;
            if (l < r * r) {
              if (x3 === 0) x3 = jiggle_default(random), l += x3 * x3;
              if (y3 === 0) y3 = jiggle_default(random), l += y3 * y3;
              l = (r - (l = Math.sqrt(l))) / l * strength;
              node.vx += (x3 *= l) * (r = (rj *= rj) / (ri2 + rj));
              node.vy += (y3 *= l) * r;
              data.vx -= x3 * (r = 1 - r);
              data.vy -= y3 * r;
            }
          }
          return;
        }
        return x0 > xi + r || x1 < xi - r || y0 > yi + r || y1 < yi - r;
      }
    }
    function prepare(quad) {
      if (quad.data) return quad.r = radii[quad.data.index];
      for (var i = quad.r = 0; i < 4; ++i) {
        if (quad[i] && quad[i].r > quad.r) {
          quad.r = quad[i].r;
        }
      }
    }
    function initialize() {
      if (!nodes) return;
      var i, n = nodes.length, node;
      radii = new Array(n);
      for (i = 0; i < n; ++i) node = nodes[i], radii[node.index] = +radius(node, i, nodes);
    }
    force.initialize = function(_nodes, _random) {
      nodes = _nodes;
      random = _random;
      initialize();
    };
    force.iterations = function(_) {
      return arguments.length ? (iterations = +_, force) : iterations;
    };
    force.strength = function(_) {
      return arguments.length ? (strength = +_, force) : strength;
    };
    force.radius = function(_) {
      return arguments.length ? (radius = typeof _ === "function" ? _ : constant_default4(+_), initialize(), force) : radius;
    };
    return force;
  }

  // node_modules/d3-force/src/link.js
  function index(d) {
    return d.index;
  }
  function find2(nodeById, nodeId) {
    var node = nodeById.get(nodeId);
    if (!node) throw new Error("node not found: " + nodeId);
    return node;
  }
  function link_default(links) {
    var id2 = index, strength = defaultStrength, strengths, distance = constant_default4(30), distances, nodes, count, bias, random, iterations = 1;
    if (links == null) links = [];
    function defaultStrength(link) {
      return 1 / Math.min(count[link.source.index], count[link.target.index]);
    }
    function force(alpha) {
      for (var k = 0, n = links.length; k < iterations; ++k) {
        for (var i = 0, link, source, target, x3, y3, l, b; i < n; ++i) {
          link = links[i], source = link.source, target = link.target;
          x3 = target.x + target.vx - source.x - source.vx || jiggle_default(random);
          y3 = target.y + target.vy - source.y - source.vy || jiggle_default(random);
          l = Math.sqrt(x3 * x3 + y3 * y3);
          l = (l - distances[i]) / l * alpha * strengths[i];
          x3 *= l, y3 *= l;
          target.vx -= x3 * (b = bias[i]);
          target.vy -= y3 * b;
          source.vx += x3 * (b = 1 - b);
          source.vy += y3 * b;
        }
      }
    }
    function initialize() {
      if (!nodes) return;
      var i, n = nodes.length, m2 = links.length, nodeById = new Map(nodes.map((d, i2) => [id2(d, i2, nodes), d])), link;
      for (i = 0, count = new Array(n); i < m2; ++i) {
        link = links[i], link.index = i;
        if (typeof link.source !== "object") link.source = find2(nodeById, link.source);
        if (typeof link.target !== "object") link.target = find2(nodeById, link.target);
        count[link.source.index] = (count[link.source.index] || 0) + 1;
        count[link.target.index] = (count[link.target.index] || 0) + 1;
      }
      for (i = 0, bias = new Array(m2); i < m2; ++i) {
        link = links[i], bias[i] = count[link.source.index] / (count[link.source.index] + count[link.target.index]);
      }
      strengths = new Array(m2), initializeStrength();
      distances = new Array(m2), initializeDistance();
    }
    function initializeStrength() {
      if (!nodes) return;
      for (var i = 0, n = links.length; i < n; ++i) {
        strengths[i] = +strength(links[i], i, links);
      }
    }
    function initializeDistance() {
      if (!nodes) return;
      for (var i = 0, n = links.length; i < n; ++i) {
        distances[i] = +distance(links[i], i, links);
      }
    }
    force.initialize = function(_nodes, _random) {
      nodes = _nodes;
      random = _random;
      initialize();
    };
    force.links = function(_) {
      return arguments.length ? (links = _, initialize(), force) : links;
    };
    force.id = function(_) {
      return arguments.length ? (id2 = _, force) : id2;
    };
    force.iterations = function(_) {
      return arguments.length ? (iterations = +_, force) : iterations;
    };
    force.strength = function(_) {
      return arguments.length ? (strength = typeof _ === "function" ? _ : constant_default4(+_), initializeStrength(), force) : strength;
    };
    force.distance = function(_) {
      return arguments.length ? (distance = typeof _ === "function" ? _ : constant_default4(+_), initializeDistance(), force) : distance;
    };
    return force;
  }

  // node_modules/d3-force/src/lcg.js
  var a = 1664525;
  var c = 1013904223;
  var m = 4294967296;
  function lcg_default() {
    let s = 1;
    return () => (s = (a * s + c) % m) / m;
  }

  // node_modules/d3-force/src/simulation.js
  function x2(d) {
    return d.x;
  }
  function y2(d) {
    return d.y;
  }
  var initialRadius = 10;
  var initialAngle = Math.PI * (3 - Math.sqrt(5));
  function simulation_default(nodes) {
    var simulation2, alpha = 1, alphaMin = 1e-3, alphaDecay = 1 - Math.pow(alphaMin, 1 / 300), alphaTarget = 0, velocityDecay = 0.6, forces = /* @__PURE__ */ new Map(), stepper = timer(step), event = dispatch_default("tick", "end"), random = lcg_default();
    if (nodes == null) nodes = [];
    function step() {
      tick();
      event.call("tick", simulation2);
      if (alpha < alphaMin) {
        stepper.stop();
        event.call("end", simulation2);
      }
    }
    function tick(iterations) {
      var i, n = nodes.length, node;
      if (iterations === void 0) iterations = 1;
      for (var k = 0; k < iterations; ++k) {
        alpha += (alphaTarget - alpha) * alphaDecay;
        forces.forEach(function(force) {
          force(alpha);
        });
        for (i = 0; i < n; ++i) {
          node = nodes[i];
          if (node.fx == null) node.x += node.vx *= velocityDecay;
          else node.x = node.fx, node.vx = 0;
          if (node.fy == null) node.y += node.vy *= velocityDecay;
          else node.y = node.fy, node.vy = 0;
        }
      }
      return simulation2;
    }
    function initializeNodes() {
      for (var i = 0, n = nodes.length, node; i < n; ++i) {
        node = nodes[i], node.index = i;
        if (node.fx != null) node.x = node.fx;
        if (node.fy != null) node.y = node.fy;
        if (isNaN(node.x) || isNaN(node.y)) {
          var radius = initialRadius * Math.sqrt(0.5 + i), angle = i * initialAngle;
          node.x = radius * Math.cos(angle);
          node.y = radius * Math.sin(angle);
        }
        if (isNaN(node.vx) || isNaN(node.vy)) {
          node.vx = node.vy = 0;
        }
      }
    }
    function initializeForce(force) {
      if (force.initialize) force.initialize(nodes, random);
      return force;
    }
    initializeNodes();
    return simulation2 = {
      tick,
      restart: function() {
        return stepper.restart(step), simulation2;
      },
      stop: function() {
        return stepper.stop(), simulation2;
      },
      nodes: function(_) {
        return arguments.length ? (nodes = _, initializeNodes(), forces.forEach(initializeForce), simulation2) : nodes;
      },
      alpha: function(_) {
        return arguments.length ? (alpha = +_, simulation2) : alpha;
      },
      alphaMin: function(_) {
        return arguments.length ? (alphaMin = +_, simulation2) : alphaMin;
      },
      alphaDecay: function(_) {
        return arguments.length ? (alphaDecay = +_, simulation2) : +alphaDecay;
      },
      alphaTarget: function(_) {
        return arguments.length ? (alphaTarget = +_, simulation2) : alphaTarget;
      },
      velocityDecay: function(_) {
        return arguments.length ? (velocityDecay = 1 - _, simulation2) : 1 - velocityDecay;
      },
      randomSource: function(_) {
        return arguments.length ? (random = _, forces.forEach(initializeForce), simulation2) : random;
      },
      force: function(name, _) {
        return arguments.length > 1 ? (_ == null ? forces.delete(name) : forces.set(name, initializeForce(_)), simulation2) : forces.get(name);
      },
      find: function(x3, y3, radius) {
        var i = 0, n = nodes.length, dx, dy, d2, node, closest;
        if (radius == null) radius = Infinity;
        else radius *= radius;
        for (i = 0; i < n; ++i) {
          node = nodes[i];
          dx = x3 - node.x;
          dy = y3 - node.y;
          d2 = dx * dx + dy * dy;
          if (d2 < radius) closest = node, radius = d2;
        }
        return closest;
      },
      on: function(name, _) {
        return arguments.length > 1 ? (event.on(name, _), simulation2) : event.on(name);
      }
    };
  }

  // node_modules/d3-force/src/manyBody.js
  function manyBody_default() {
    var nodes, node, random, alpha, strength = constant_default4(-30), strengths, distanceMin2 = 1, distanceMax2 = Infinity, theta2 = 0.81;
    function force(_) {
      var i, n = nodes.length, tree = quadtree(nodes, x2, y2).visitAfter(accumulate);
      for (alpha = _, i = 0; i < n; ++i) node = nodes[i], tree.visit(apply);
    }
    function initialize() {
      if (!nodes) return;
      var i, n = nodes.length, node2;
      strengths = new Array(n);
      for (i = 0; i < n; ++i) node2 = nodes[i], strengths[node2.index] = +strength(node2, i, nodes);
    }
    function accumulate(quad) {
      var strength2 = 0, q, c2, weight = 0, x3, y3, i;
      if (quad.length) {
        for (x3 = y3 = i = 0; i < 4; ++i) {
          if ((q = quad[i]) && (c2 = Math.abs(q.value))) {
            strength2 += q.value, weight += c2, x3 += c2 * q.x, y3 += c2 * q.y;
          }
        }
        quad.x = x3 / weight;
        quad.y = y3 / weight;
      } else {
        q = quad;
        q.x = q.data.x;
        q.y = q.data.y;
        do
          strength2 += strengths[q.data.index];
        while (q = q.next);
      }
      quad.value = strength2;
    }
    function apply(quad, x1, _, x22) {
      if (!quad.value) return true;
      var x3 = quad.x - node.x, y3 = quad.y - node.y, w = x22 - x1, l = x3 * x3 + y3 * y3;
      if (w * w / theta2 < l) {
        if (l < distanceMax2) {
          if (x3 === 0) x3 = jiggle_default(random), l += x3 * x3;
          if (y3 === 0) y3 = jiggle_default(random), l += y3 * y3;
          if (l < distanceMin2) l = Math.sqrt(distanceMin2 * l);
          node.vx += x3 * quad.value * alpha / l;
          node.vy += y3 * quad.value * alpha / l;
        }
        return true;
      } else if (quad.length || l >= distanceMax2) return;
      if (quad.data !== node || quad.next) {
        if (x3 === 0) x3 = jiggle_default(random), l += x3 * x3;
        if (y3 === 0) y3 = jiggle_default(random), l += y3 * y3;
        if (l < distanceMin2) l = Math.sqrt(distanceMin2 * l);
      }
      do
        if (quad.data !== node) {
          w = strengths[quad.data.index] * alpha / l;
          node.vx += x3 * w;
          node.vy += y3 * w;
        }
      while (quad = quad.next);
    }
    force.initialize = function(_nodes, _random) {
      nodes = _nodes;
      random = _random;
      initialize();
    };
    force.strength = function(_) {
      return arguments.length ? (strength = typeof _ === "function" ? _ : constant_default4(+_), initialize(), force) : strength;
    };
    force.distanceMin = function(_) {
      return arguments.length ? (distanceMin2 = _ * _, force) : Math.sqrt(distanceMin2);
    };
    force.distanceMax = function(_) {
      return arguments.length ? (distanceMax2 = _ * _, force) : Math.sqrt(distanceMax2);
    };
    force.theta = function(_) {
      return arguments.length ? (theta2 = _ * _, force) : Math.sqrt(theta2);
    };
    return force;
  }

  // node_modules/d3-force/src/x.js
  function x_default2(x3) {
    var strength = constant_default4(0.1), nodes, strengths, xz;
    if (typeof x3 !== "function") x3 = constant_default4(x3 == null ? 0 : +x3);
    function force(alpha) {
      for (var i = 0, n = nodes.length, node; i < n; ++i) {
        node = nodes[i], node.vx += (xz[i] - node.x) * strengths[i] * alpha;
      }
    }
    function initialize() {
      if (!nodes) return;
      var i, n = nodes.length;
      strengths = new Array(n);
      xz = new Array(n);
      for (i = 0; i < n; ++i) {
        strengths[i] = isNaN(xz[i] = +x3(nodes[i], i, nodes)) ? 0 : +strength(nodes[i], i, nodes);
      }
    }
    force.initialize = function(_) {
      nodes = _;
      initialize();
    };
    force.strength = function(_) {
      return arguments.length ? (strength = typeof _ === "function" ? _ : constant_default4(+_), initialize(), force) : strength;
    };
    force.x = function(_) {
      return arguments.length ? (x3 = typeof _ === "function" ? _ : constant_default4(+_), initialize(), force) : x3;
    };
    return force;
  }

  // node_modules/d3-force/src/y.js
  function y_default2(y3) {
    var strength = constant_default4(0.1), nodes, strengths, yz;
    if (typeof y3 !== "function") y3 = constant_default4(y3 == null ? 0 : +y3);
    function force(alpha) {
      for (var i = 0, n = nodes.length, node; i < n; ++i) {
        node = nodes[i], node.vy += (yz[i] - node.y) * strengths[i] * alpha;
      }
    }
    function initialize() {
      if (!nodes) return;
      var i, n = nodes.length;
      strengths = new Array(n);
      yz = new Array(n);
      for (i = 0; i < n; ++i) {
        strengths[i] = isNaN(yz[i] = +y3(nodes[i], i, nodes)) ? 0 : +strength(nodes[i], i, nodes);
      }
    }
    force.initialize = function(_) {
      nodes = _;
      initialize();
    };
    force.strength = function(_) {
      return arguments.length ? (strength = typeof _ === "function" ? _ : constant_default4(+_), initialize(), force) : strength;
    };
    force.y = function(_) {
      return arguments.length ? (y3 = typeof _ === "function" ? _ : constant_default4(+_), initialize(), force) : y3;
    };
    return force;
  }

  // node_modules/d3-zoom/src/transform.js
  function Transform(k, x3, y3) {
    this.k = k;
    this.x = x3;
    this.y = y3;
  }
  Transform.prototype = {
    constructor: Transform,
    scale: function(k) {
      return k === 1 ? this : new Transform(this.k * k, this.x, this.y);
    },
    translate: function(x3, y3) {
      return x3 === 0 & y3 === 0 ? this : new Transform(this.k, this.x + this.k * x3, this.y + this.k * y3);
    },
    apply: function(point) {
      return [point[0] * this.k + this.x, point[1] * this.k + this.y];
    },
    applyX: function(x3) {
      return x3 * this.k + this.x;
    },
    applyY: function(y3) {
      return y3 * this.k + this.y;
    },
    invert: function(location) {
      return [(location[0] - this.x) / this.k, (location[1] - this.y) / this.k];
    },
    invertX: function(x3) {
      return (x3 - this.x) / this.k;
    },
    invertY: function(y3) {
      return (y3 - this.y) / this.k;
    },
    rescaleX: function(x3) {
      return x3.copy().domain(x3.range().map(this.invertX, this).map(x3.invert, x3));
    },
    rescaleY: function(y3) {
      return y3.copy().domain(y3.range().map(this.invertY, this).map(y3.invert, y3));
    },
    toString: function() {
      return "translate(" + this.x + "," + this.y + ") scale(" + this.k + ")";
    }
  };
  var identity2 = new Transform(1, 0, 0);
  transform.prototype = Transform.prototype;
  function transform(node) {
    while (!node.__zoom) if (!(node = node.parentNode)) return identity2;
    return node.__zoom;
  }

  // src/webview/physics/PhysicsConfig.ts
  var DEFAULT_PHYSICS_CONFIG = {
    chargeStrength: -600,
    // Increased repulsion for larger nodes
    chargeDistanceMax: 2e3,
    // Effectively global charge force for more natural layout
    chargeTheta: 0.9,
    // Default Barnes-Hut accuracy
    collisionPadding: 35,
    // Increased padding for 100:50:20 node sizes
    linkDistance: 150,
    // Longer edges to accommodate larger nodes
    linkStrength: 1,
    velocityDecay: 0.8,
    // Increased friction for faster settling
    alphaDecay: 0.0228,
    // D3 default - allows more time for layout to settle properly
    xStrength: 2e-3,
    // Gentle gravity to prevent disconnected subgraphs from drifting
    yStrength: 2e-3
    // Gentle gravity to prevent disconnected subgraphs from drifting
  };

  // src/webview/components/D3View/constants.ts
  var REGULAR_NODE_BASE_SIZE = 40;
  var REGULAR_NODE_HALF_SIZE = REGULAR_NODE_BASE_SIZE / 2;
  var REGULAR_NODE_CORNER_RADIUS = Math.round(REGULAR_NODE_BASE_SIZE * 0.22);
  var REGULAR_NODE_ICON_SCALE = REGULAR_NODE_BASE_SIZE / 60;
  var REGULAR_NODE_PIN_OUTER_STROKE_WIDTH = Math.max(1, Math.round(REGULAR_NODE_BASE_SIZE * 0.06));
  var REGULAR_NODE_PIN_INNER_STROKE_WIDTH = Math.max(1, Math.round(REGULAR_NODE_BASE_SIZE * 0.07));
  var DIRECTORY_PIN_ICON_OFFSET_Y = Math.round(REGULAR_NODE_BASE_SIZE * 0.1);
  var FILE_PIN_ICON_OFFSET_Y = Math.round(REGULAR_NODE_BASE_SIZE * 0.06);
  var REGULAR_NODE_LABEL_MIN_SCREEN_SIZE = Math.max(9, Math.round(REGULAR_NODE_BASE_SIZE * 0.25));
  var REGULAR_NODE_LABEL_TARGET_SCREEN_SIZE = Math.round(REGULAR_NODE_BASE_SIZE * 0.3);
  var REGULAR_NODE_LABEL_MAX_SCREEN_SIZE = Math.round(REGULAR_NODE_BASE_SIZE * 0.32);

  // src/webview/components/D3View/layouts/physicsHelpers.ts
  var LINK_PADDING = {
    domainDomain: 600,
    domainOther: 345,
    subdomainOther: 253,
    default: 186
  };
  var BASE_CHARGE = {
    Domain: -4e3,
    Subdomain: -2240,
    default: -1250
  };
  var BASE_LINK_STRENGTH = {
    domainDomain: 0.04,
    domainOther: 0.21,
    default: 0.42
  };
  var COLLISION_PADDING = {
    Domain: 60,
    Subdomain: 50,
    default: 42
  };
  var BASE_CENTER_STRENGTH = 5e-4;
  function getDensityFactor(nodeCount) {
    return nodeCount > 0 ? Math.min(nodeCount / 100, 1) : 0;
  }
  function getCollisionPadding(node) {
    if (node && node.type === "Domain") {
      return COLLISION_PADDING.Domain;
    }
    if (node && node.type === "Subdomain") {
      return COLLISION_PADDING.Subdomain;
    }
    return COLLISION_PADDING.default;
  }
  function computeHalfDiagonal(node) {
    if (!node) {
      return REGULAR_NODE_BASE_SIZE / 2;
    }
    const widthValue = typeof node.width === "number" ? node.width : void 0;
    const heightValue = typeof node.height === "number" ? node.height : void 0;
    if (widthValue !== void 0 && heightValue !== void 0) {
      const halfWidth = widthValue / 2;
      const halfHeight = heightValue / 2;
      return Math.sqrt(halfWidth * halfWidth + halfHeight * halfHeight);
    }
    const fallbackSize = typeof node.physicsWeight === "number" ? node.physicsWeight : REGULAR_NODE_BASE_SIZE;
    return fallbackSize / 2;
  }
  function computeCollisionRadius(node) {
    const padding = getCollisionPadding(node);
    if (!node) {
      return REGULAR_NODE_BASE_SIZE / 2 + padding;
    }
    const widthValue = typeof node.width === "number" ? node.width : void 0;
    const heightValue = typeof node.height === "number" ? node.height : void 0;
    if (widthValue !== void 0 && heightValue !== void 0) {
      const largerHalf = widthValue > heightValue ? widthValue / 2 : heightValue / 2;
      return largerHalf + padding;
    }
    const fallbackSize = typeof node.physicsWeight === "number" ? node.physicsWeight : REGULAR_NODE_BASE_SIZE;
    return fallbackSize / 2 + padding;
  }
  function computeLinkDistance(source, target, labelWidth, nodeCount) {
    if (!source || !target) {
      return 150;
    }
    const sourceRadius = computeHalfDiagonal(source);
    const targetRadius = computeHalfDiagonal(target);
    let padding = LINK_PADDING.default;
    const sourceType = source.type;
    const targetType = target.type;
    const isDomainDomain = sourceType === "Domain" && targetType === "Domain";
    const isDomainOther = !isDomainDomain && (sourceType === "Domain" || targetType === "Domain");
    const isSubdomainPresent = sourceType === "Subdomain" || targetType === "Subdomain";
    if (isDomainDomain) {
      padding = LINK_PADDING.domainDomain;
    } else if (isDomainOther) {
      padding = LINK_PADDING.domainOther;
    } else if (isSubdomainPresent) {
      padding = LINK_PADDING.subdomainOther;
    }
    let distance = sourceRadius + targetRadius + padding;
    if (isDomainDomain && typeof labelWidth === "number" && labelWidth > 0) {
      distance += labelWidth;
    }
    let sparseBoost = 0;
    if (typeof nodeCount === "number" && nodeCount < 50) {
      const density = getDensityFactor(nodeCount);
      sparseBoost = padding * (1 - density) * 0.9;
    }
    return distance + sparseBoost;
  }
  function computeChargeStrength(node, nodeCount) {
    const isFrozen = node && node.isHiddenByFocusMode && node.fx !== null && node.fx !== void 0;
    if (isFrozen) {
      return -1;
    }
    const density = getDensityFactor(nodeCount);
    let baseCharge = BASE_CHARGE.default;
    if (node && node.type === "Domain") {
      baseCharge = BASE_CHARGE.Domain;
    } else if (node && node.type === "Subdomain") {
      baseCharge = BASE_CHARGE.Subdomain;
    }
    const densityMultiplier = 1.3 - density * 0.3;
    const visibleDegree = node && typeof node.visibleDegree === "number" ? node.visibleDegree : 0;
    const degreeBoost = 1 + Math.log10(visibleDegree + 1) * 0.1;
    return baseCharge * densityMultiplier * degreeBoost;
  }
  function computeLinkStrength(source, target, nodeCount) {
    const density = getDensityFactor(nodeCount);
    let baseStrength = BASE_LINK_STRENGTH.default;
    const sourceType = source ? source.type : void 0;
    const targetType = target ? target.type : void 0;
    const isDomainDomain = sourceType === "Domain" && targetType === "Domain";
    const isDomainOther = !isDomainDomain && (sourceType === "Domain" || targetType === "Domain");
    if (isDomainDomain) {
      baseStrength = BASE_LINK_STRENGTH.domainDomain;
    } else if (isDomainOther) {
      baseStrength = BASE_LINK_STRENGTH.domainOther;
    }
    const densityMultiplier = 0.4 + density * 0.3;
    return baseStrength * densityMultiplier;
  }
  function computeCenterStrength(nodeCount) {
    const density = getDensityFactor(nodeCount);
    return BASE_CENTER_STRENGTH * (1 + density * 2);
  }

  // src/webview/components/D3View/layouts/force-worker.ts
  var simulation = null;
  var currentPhysicsConfig = DEFAULT_PHYSICS_CONFIG;
  var lastSentPositions = /* @__PURE__ */ new Map();
  var positionChangeThreshold = 1;
  var positionBuffer = /* @__PURE__ */ new Map();
  var messageScheduled = false;
  var batchTimer = null;
  var physicsCooldownTimer = null;
  var simulationWidth = 800;
  var simulationHeight = 600;
  function getChangedNodes(nodes) {
    const changedNodes = [];
    for (const node of nodes) {
      const lastPos = lastSentPositions.get(node.id);
      if (!lastPos || Math.abs(node.x - lastPos.x) > positionChangeThreshold || Math.abs(node.y - lastPos.y) > positionChangeThreshold) {
        changedNodes.push({ id: node.id, x: node.x, y: node.y });
        lastSentPositions.set(node.id, { x: node.x, y: node.y });
      }
    }
    return changedNodes;
  }
  function sendBatchedUpdate() {
    if (positionBuffer.size > 0) {
      const nodes = Array.from(positionBuffer.entries()).map(([id2, { x: x3, y: y3 }]) => ({ id: id2, x: x3, y: y3 }));
      const changedNodeIds = nodes.map((n) => n.id);
      postMessage({
        type: "tick",
        nodes,
        changedNodeIds,
        isDelta: true
      });
      positionBuffer.clear();
    }
    messageScheduled = false;
    batchTimer = null;
  }
  function createWorkerSimulation(width, height) {
    const config = currentPhysicsConfig;
    simulationWidth = width;
    simulationHeight = height;
    const sim = simulation_default().alphaDecay(config.alphaDecay).velocityDecay(config.velocityDecay).force("charge", manyBody_default().strength((node) => {
      const nodes = sim.nodes();
      return computeChargeStrength(node, nodes.length);
    }).distanceMax(config.chargeDistanceMax).theta(config.chargeTheta)).force("center", center_default(width / 2, height / 2)).force("x", x_default2(width / 2).strength(() => {
      const nodes = sim.nodes();
      return computeCenterStrength(nodes.length);
    })).force("y", y_default2(height / 2).strength(() => {
      const nodes = sim.nodes();
      return computeCenterStrength(nodes.length);
    })).force("collision", collide_default().radius((node) => computeCollisionRadius(node)));
    return sim;
  }
  function applyPhysicsConfigToSimulation(sim, config) {
    sim.alphaDecay(config.alphaDecay).velocityDecay(config.velocityDecay);
    const chargeForce = sim.force("charge");
    if (chargeForce) {
      chargeForce.strength((node) => {
        const nodes = sim.nodes();
        return computeChargeStrength(node, nodes.length);
      }).distanceMax(config.chargeDistanceMax).theta(config.chargeTheta);
    }
    const collisionForce = sim.force("collision");
    if (collisionForce) {
      collisionForce.radius((node) => computeCollisionRadius(node));
    }
    const linkForce = sim.force("link");
    if (linkForce) {
      linkForce.strength((link) => {
        const nodes = sim.nodes();
        const source = typeof link.source === "object" ? link.source : findNodeById(nodes, link.source);
        const target = typeof link.target === "object" ? link.target : findNodeById(nodes, link.target);
        return computeLinkStrength(source, target, nodes.length);
      }).distance((link) => {
        const nodes = sim.nodes();
        const source = typeof link.source === "object" ? link.source : findNodeById(nodes, link.source);
        const target = typeof link.target === "object" ? link.target : findNodeById(nodes, link.target);
        const labelWidth = typeof link.labelWidth === "number" ? link.labelWidth : void 0;
        return computeLinkDistance(source, target, labelWidth, nodes.length);
      });
    }
    sim.force("x", x_default2(simulationWidth / 2).strength(() => {
      const nodes = sim.nodes();
      return computeCenterStrength(nodes.length);
    }));
    sim.force("y", y_default2(simulationHeight / 2).strength(() => {
      const nodes = sim.nodes();
      return computeCenterStrength(nodes.length);
    }));
  }
  self.onmessage = function(event) {
    const { type: type2, nodes, links, updatedNodes, width, height, alpha, nodeId, fx, fy, nodeIds, dx, dy, active, config, layoutHint } = event.data;
    if (type2 !== "updateNodePositions" || Math.random() < 0.1) {
    }
    try {
      switch (type2) {
        case "start":
          if (!nodes || !links || width === void 0 || height === void 0) {
            postMessage({
              type: "error",
              error: "Missing required data for simulation start"
            });
            return;
          }
          lastSentPositions.clear();
          positionBuffer.clear();
          if (batchTimer) {
            clearTimeout(batchTimer);
            batchTimer = null;
          }
          messageScheduled = false;
          simulation = createWorkerSimulation(width, height);
          simulation.nodes(nodes);
          const linkForce = link_default(links).id((link) => link.id).strength((link) => {
            const simNodes2 = simulation.nodes();
            const source = typeof link.source === "object" ? link.source : findNodeById(simNodes2, link.source);
            const target = typeof link.target === "object" ? link.target : findNodeById(simNodes2, link.target);
            return computeLinkStrength(source, target, simNodes2.length);
          }).distance((link) => {
            const simNodes2 = simulation.nodes();
            const source = typeof link.source === "object" ? link.source : findNodeById(simNodes2, link.source);
            const target = typeof link.target === "object" ? link.target : findNodeById(simNodes2, link.target);
            const labelWidth = typeof link.labelWidth === "number" ? link.labelWidth : void 0;
            return computeLinkDistance(source, target, labelWidth, simNodes2.length);
          });
          simulation.force("link", linkForce);
          {
            let tickCount = 0;
            simulation.on("tick", () => {
              if (!simulation) return;
              tickCount++;
              const nodes2 = simulation.nodes();
              if (tickCount <= 3) {
              }
              const changedNodes = getChangedNodes(nodes2);
              changedNodes.forEach((node) => {
                positionBuffer.set(node.id, { x: node.x, y: node.y });
              });
              if (!messageScheduled && positionBuffer.size > 0) {
                messageScheduled = true;
                batchTimer = setTimeout(sendBatchedUpdate, 16);
              }
            });
            simulation.on("end", () => {
              if (positionBuffer.size > 0) {
                sendBatchedUpdate();
              }
              postMessage({ type: "end" });
            });
            if (layoutHint === "incremental") {
              simulation.alpha(0.3).restart();
            } else if (layoutHint === "focusModeToggle") {
              simulation.alpha(0.1).restart();
            } else {
              simulation.alpha(1).restart();
            }
          }
          break;
        case "startDrag":
          if (!simulation) return;
          const startNode = simulation.nodes().find((n) => n.id === nodeId);
          if (startNode) {
            startNode.fx = fx;
            startNode.fy = fy;
            if (!active) {
              simulation.alphaTarget(0.3).restart();
            } else {
              simulation.alphaTarget(0.3);
            }
          }
          break;
        case "updateDragPosition":
          if (!simulation) return;
          const dragNode = simulation.nodes().find((n) => n.id === nodeId);
          if (dragNode) {
            dragNode.fx = fx;
            dragNode.fy = fy;
            if (simulation.alpha() < 0.01) {
              simulation.alpha(0.1);
            }
          }
          break;
        case "endDrag":
          if (!simulation) return;
          const endNode = simulation.nodes().find((n) => n.id === nodeId);
          if (endNode) {
            endNode.fx = null;
            endNode.fy = null;
            if (!active) {
              simulation.alphaTarget(0);
            } else {
              simulation.alphaTarget(0.1).restart();
              simulation.alphaTarget(0);
            }
          }
          break;
        case "startContainerDrag":
          if (!simulation || !nodeIds) return;
          const containerNodes = simulation.nodes().filter((n) => nodeIds.includes(n.id));
          containerNodes.forEach((node) => {
            node.fx = node.x;
            node.fy = node.y;
          });
          simulation.alphaTarget(0.3).restart();
          break;
        case "updateContainerDrag":
          if (!simulation || !nodeIds || dx === void 0 || dy === void 0) return;
          const dragContainerNodes = simulation.nodes().filter((n) => nodeIds.includes(n.id));
          dragContainerNodes.forEach((node) => {
            if (node.fx !== null && node.fy !== null) {
              node.fx = (node.fx || node.x) + dx;
              node.fy = (node.fy || node.y) + dy;
            }
          });
          break;
        case "endContainerDrag":
          if (!simulation || !nodeIds) return;
          const endContainerNodes = simulation.nodes().filter((n) => nodeIds.includes(n.id));
          endContainerNodes.forEach((node) => {
            node.fx = null;
            node.fy = null;
          });
          simulation.alphaTarget(0);
          simulation.alpha(0.1).restart();
          break;
        case "updateNodePositions":
          if (Math.random() < 0.1) {
          }
          if (!simulation) {
            return;
          }
          if (!updatedNodes) {
            postMessage({
              type: "error",
              error: "No active simulation or missing node data"
            });
            return;
          }
          const simNodes = simulation.nodes();
          if (Math.random() < 0.1) {
          }
          updatedNodes.forEach(({ id: id2, fx: fx2, fy: fy2 }) => {
            const node = simNodes.find((n) => n.id === id2);
            if (node) {
              if (Math.random() < 0.05) {
              }
              node.fx = fx2;
              node.fy = fy2;
            } else {
            }
          });
          simulation.alpha(0.05).restart();
          break;
        case "reheat":
          if (!simulation) return;
          const reheatingAlpha = alpha || 0.1;
          simulation.alpha(reheatingAlpha).restart();
          break;
        case "updatePhysics":
          if (!config) return;
          currentPhysicsConfig = config;
          if (simulation) {
            applyPhysicsConfigToSimulation(simulation, currentPhysicsConfig);
            if (physicsCooldownTimer) {
              clearTimeout(physicsCooldownTimer);
              physicsCooldownTimer = null;
            }
            simulation.alphaTarget(0.3).restart();
            physicsCooldownTimer = setTimeout(() => {
              if (simulation) {
                simulation.alphaTarget(0);
              }
              physicsCooldownTimer = null;
            }, 1500);
          }
          break;
        case "stop":
          if (simulation) {
            simulation.on("tick", null);
            simulation.on("end", null);
            simulation.stop();
            simulation = null;
          }
          if (physicsCooldownTimer) {
            clearTimeout(physicsCooldownTimer);
            physicsCooldownTimer = null;
          }
          break;
        default:
          postMessage({
            type: "error",
            error: `Unknown message type: ${type2}`
          });
      }
    } catch (error) {
      postMessage({
        type: "error",
        error: error instanceof Error ? error.message : "Unknown error in worker"
      });
    }
  };
  function findNodeById(nodes, id2) {
    for (let i = 0; i < nodes.length; i += 1) {
      const candidate = nodes[i];
      if (candidate && candidate.id === id2) {
        return candidate;
      }
    }
    return void 0;
  }
})();
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLWRpc3BhdGNoL3NyYy9kaXNwYXRjaC5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtc2VsZWN0aW9uL3NyYy9uYW1lc3BhY2VzLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1zZWxlY3Rpb24vc3JjL25hbWVzcGFjZS5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtc2VsZWN0aW9uL3NyYy9jcmVhdG9yLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1zZWxlY3Rpb24vc3JjL3NlbGVjdG9yLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1zZWxlY3Rpb24vc3JjL3NlbGVjdGlvbi9zZWxlY3QuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXNlbGVjdGlvbi9zcmMvYXJyYXkuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXNlbGVjdGlvbi9zcmMvc2VsZWN0b3JBbGwuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXNlbGVjdGlvbi9zcmMvc2VsZWN0aW9uL3NlbGVjdEFsbC5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtc2VsZWN0aW9uL3NyYy9tYXRjaGVyLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1zZWxlY3Rpb24vc3JjL3NlbGVjdGlvbi9zZWxlY3RDaGlsZC5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtc2VsZWN0aW9uL3NyYy9zZWxlY3Rpb24vc2VsZWN0Q2hpbGRyZW4uanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXNlbGVjdGlvbi9zcmMvc2VsZWN0aW9uL2ZpbHRlci5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtc2VsZWN0aW9uL3NyYy9zZWxlY3Rpb24vc3BhcnNlLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1zZWxlY3Rpb24vc3JjL3NlbGVjdGlvbi9lbnRlci5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtc2VsZWN0aW9uL3NyYy9jb25zdGFudC5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtc2VsZWN0aW9uL3NyYy9zZWxlY3Rpb24vZGF0YS5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtc2VsZWN0aW9uL3NyYy9zZWxlY3Rpb24vZXhpdC5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtc2VsZWN0aW9uL3NyYy9zZWxlY3Rpb24vam9pbi5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtc2VsZWN0aW9uL3NyYy9zZWxlY3Rpb24vbWVyZ2UuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXNlbGVjdGlvbi9zcmMvc2VsZWN0aW9uL29yZGVyLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1zZWxlY3Rpb24vc3JjL3NlbGVjdGlvbi9zb3J0LmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1zZWxlY3Rpb24vc3JjL3NlbGVjdGlvbi9jYWxsLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1zZWxlY3Rpb24vc3JjL3NlbGVjdGlvbi9ub2Rlcy5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtc2VsZWN0aW9uL3NyYy9zZWxlY3Rpb24vbm9kZS5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtc2VsZWN0aW9uL3NyYy9zZWxlY3Rpb24vc2l6ZS5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtc2VsZWN0aW9uL3NyYy9zZWxlY3Rpb24vZW1wdHkuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXNlbGVjdGlvbi9zcmMvc2VsZWN0aW9uL2VhY2guanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXNlbGVjdGlvbi9zcmMvc2VsZWN0aW9uL2F0dHIuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXNlbGVjdGlvbi9zcmMvd2luZG93LmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1zZWxlY3Rpb24vc3JjL3NlbGVjdGlvbi9zdHlsZS5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtc2VsZWN0aW9uL3NyYy9zZWxlY3Rpb24vcHJvcGVydHkuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXNlbGVjdGlvbi9zcmMvc2VsZWN0aW9uL2NsYXNzZWQuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXNlbGVjdGlvbi9zcmMvc2VsZWN0aW9uL3RleHQuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXNlbGVjdGlvbi9zcmMvc2VsZWN0aW9uL2h0bWwuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXNlbGVjdGlvbi9zcmMvc2VsZWN0aW9uL3JhaXNlLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1zZWxlY3Rpb24vc3JjL3NlbGVjdGlvbi9sb3dlci5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtc2VsZWN0aW9uL3NyYy9zZWxlY3Rpb24vYXBwZW5kLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1zZWxlY3Rpb24vc3JjL3NlbGVjdGlvbi9pbnNlcnQuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXNlbGVjdGlvbi9zcmMvc2VsZWN0aW9uL3JlbW92ZS5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtc2VsZWN0aW9uL3NyYy9zZWxlY3Rpb24vY2xvbmUuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXNlbGVjdGlvbi9zcmMvc2VsZWN0aW9uL2RhdHVtLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1zZWxlY3Rpb24vc3JjL3NlbGVjdGlvbi9vbi5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtc2VsZWN0aW9uL3NyYy9zZWxlY3Rpb24vZGlzcGF0Y2guanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXNlbGVjdGlvbi9zcmMvc2VsZWN0aW9uL2l0ZXJhdG9yLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1zZWxlY3Rpb24vc3JjL3NlbGVjdGlvbi9pbmRleC5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtY29sb3Ivc3JjL2RlZmluZS5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtY29sb3Ivc3JjL2NvbG9yLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1pbnRlcnBvbGF0ZS9zcmMvYmFzaXMuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLWludGVycG9sYXRlL3NyYy9iYXNpc0Nsb3NlZC5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtaW50ZXJwb2xhdGUvc3JjL2NvbnN0YW50LmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1pbnRlcnBvbGF0ZS9zcmMvY29sb3IuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLWludGVycG9sYXRlL3NyYy9yZ2IuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLWludGVycG9sYXRlL3NyYy9udW1iZXIuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLWludGVycG9sYXRlL3NyYy9zdHJpbmcuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLWludGVycG9sYXRlL3NyYy90cmFuc2Zvcm0vZGVjb21wb3NlLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1pbnRlcnBvbGF0ZS9zcmMvdHJhbnNmb3JtL3BhcnNlLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1pbnRlcnBvbGF0ZS9zcmMvdHJhbnNmb3JtL2luZGV4LmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy10aW1lci9zcmMvdGltZXIuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXRpbWVyL3NyYy90aW1lb3V0LmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy10cmFuc2l0aW9uL3NyYy90cmFuc2l0aW9uL3NjaGVkdWxlLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy10cmFuc2l0aW9uL3NyYy9pbnRlcnJ1cHQuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXRyYW5zaXRpb24vc3JjL3NlbGVjdGlvbi9pbnRlcnJ1cHQuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXRyYW5zaXRpb24vc3JjL3RyYW5zaXRpb24vdHdlZW4uanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXRyYW5zaXRpb24vc3JjL3RyYW5zaXRpb24vaW50ZXJwb2xhdGUuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXRyYW5zaXRpb24vc3JjL3RyYW5zaXRpb24vYXR0ci5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtdHJhbnNpdGlvbi9zcmMvdHJhbnNpdGlvbi9hdHRyVHdlZW4uanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXRyYW5zaXRpb24vc3JjL3RyYW5zaXRpb24vZGVsYXkuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXRyYW5zaXRpb24vc3JjL3RyYW5zaXRpb24vZHVyYXRpb24uanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXRyYW5zaXRpb24vc3JjL3RyYW5zaXRpb24vZWFzZS5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtdHJhbnNpdGlvbi9zcmMvdHJhbnNpdGlvbi9lYXNlVmFyeWluZy5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtdHJhbnNpdGlvbi9zcmMvdHJhbnNpdGlvbi9maWx0ZXIuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXRyYW5zaXRpb24vc3JjL3RyYW5zaXRpb24vbWVyZ2UuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXRyYW5zaXRpb24vc3JjL3RyYW5zaXRpb24vb24uanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXRyYW5zaXRpb24vc3JjL3RyYW5zaXRpb24vcmVtb3ZlLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy10cmFuc2l0aW9uL3NyYy90cmFuc2l0aW9uL3NlbGVjdC5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtdHJhbnNpdGlvbi9zcmMvdHJhbnNpdGlvbi9zZWxlY3RBbGwuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXRyYW5zaXRpb24vc3JjL3RyYW5zaXRpb24vc2VsZWN0aW9uLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy10cmFuc2l0aW9uL3NyYy90cmFuc2l0aW9uL3N0eWxlLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy10cmFuc2l0aW9uL3NyYy90cmFuc2l0aW9uL3N0eWxlVHdlZW4uanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXRyYW5zaXRpb24vc3JjL3RyYW5zaXRpb24vdGV4dC5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtdHJhbnNpdGlvbi9zcmMvdHJhbnNpdGlvbi90ZXh0VHdlZW4uanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXRyYW5zaXRpb24vc3JjL3RyYW5zaXRpb24vdHJhbnNpdGlvbi5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtdHJhbnNpdGlvbi9zcmMvdHJhbnNpdGlvbi9lbmQuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXRyYW5zaXRpb24vc3JjL3RyYW5zaXRpb24vaW5kZXguanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLWVhc2Uvc3JjL2N1YmljLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy10cmFuc2l0aW9uL3NyYy9zZWxlY3Rpb24vdHJhbnNpdGlvbi5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtdHJhbnNpdGlvbi9zcmMvc2VsZWN0aW9uL2luZGV4LmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1icnVzaC9zcmMvYnJ1c2guanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLWZvcmNlL3NyYy9jZW50ZXIuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXF1YWR0cmVlL3NyYy9hZGQuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXF1YWR0cmVlL3NyYy9jb3Zlci5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtcXVhZHRyZWUvc3JjL2RhdGEuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXF1YWR0cmVlL3NyYy9leHRlbnQuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXF1YWR0cmVlL3NyYy9xdWFkLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1xdWFkdHJlZS9zcmMvZmluZC5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtcXVhZHRyZWUvc3JjL3JlbW92ZS5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtcXVhZHRyZWUvc3JjL3Jvb3QuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXF1YWR0cmVlL3NyYy9zaXplLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1xdWFkdHJlZS9zcmMvdmlzaXQuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXF1YWR0cmVlL3NyYy92aXNpdEFmdGVyLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1xdWFkdHJlZS9zcmMveC5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtcXVhZHRyZWUvc3JjL3kuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLXF1YWR0cmVlL3NyYy9xdWFkdHJlZS5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtZm9yY2Uvc3JjL2NvbnN0YW50LmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1mb3JjZS9zcmMvamlnZ2xlLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1mb3JjZS9zcmMvY29sbGlkZS5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtZm9yY2Uvc3JjL2xpbmsuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLWZvcmNlL3NyYy9sY2cuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLWZvcmNlL3NyYy9zaW11bGF0aW9uLmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1mb3JjZS9zcmMvbWFueUJvZHkuanMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL2QzLWZvcmNlL3NyYy94LmpzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9kMy1mb3JjZS9zcmMveS5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvZDMtem9vbS9zcmMvdHJhbnNmb3JtLmpzIiwgIi4uLy4uL3NyYy93ZWJ2aWV3L3BoeXNpY3MvUGh5c2ljc0NvbmZpZy50cyIsICIuLi8uLi9zcmMvd2Vidmlldy9jb21wb25lbnRzL0QzVmlldy9jb25zdGFudHMudHMiLCAiLi4vLi4vc3JjL3dlYnZpZXcvY29tcG9uZW50cy9EM1ZpZXcvbGF5b3V0cy9waHlzaWNzSGVscGVycy50cyIsICIuLi8uLi9zcmMvd2Vidmlldy9jb21wb25lbnRzL0QzVmlldy9sYXlvdXRzL2ZvcmNlLXdvcmtlci50cyJdLAogICJzb3VyY2VzQ29udGVudCI6IFsidmFyIG5vb3AgPSB7dmFsdWU6ICgpID0+IHt9fTtcblxuZnVuY3Rpb24gZGlzcGF0Y2goKSB7XG4gIGZvciAodmFyIGkgPSAwLCBuID0gYXJndW1lbnRzLmxlbmd0aCwgXyA9IHt9LCB0OyBpIDwgbjsgKytpKSB7XG4gICAgaWYgKCEodCA9IGFyZ3VtZW50c1tpXSArIFwiXCIpIHx8ICh0IGluIF8pIHx8IC9bXFxzLl0vLnRlc3QodCkpIHRocm93IG5ldyBFcnJvcihcImlsbGVnYWwgdHlwZTogXCIgKyB0KTtcbiAgICBfW3RdID0gW107XG4gIH1cbiAgcmV0dXJuIG5ldyBEaXNwYXRjaChfKTtcbn1cblxuZnVuY3Rpb24gRGlzcGF0Y2goXykge1xuICB0aGlzLl8gPSBfO1xufVxuXG5mdW5jdGlvbiBwYXJzZVR5cGVuYW1lcyh0eXBlbmFtZXMsIHR5cGVzKSB7XG4gIHJldHVybiB0eXBlbmFtZXMudHJpbSgpLnNwbGl0KC9efFxccysvKS5tYXAoZnVuY3Rpb24odCkge1xuICAgIHZhciBuYW1lID0gXCJcIiwgaSA9IHQuaW5kZXhPZihcIi5cIik7XG4gICAgaWYgKGkgPj0gMCkgbmFtZSA9IHQuc2xpY2UoaSArIDEpLCB0ID0gdC5zbGljZSgwLCBpKTtcbiAgICBpZiAodCAmJiAhdHlwZXMuaGFzT3duUHJvcGVydHkodCkpIHRocm93IG5ldyBFcnJvcihcInVua25vd24gdHlwZTogXCIgKyB0KTtcbiAgICByZXR1cm4ge3R5cGU6IHQsIG5hbWU6IG5hbWV9O1xuICB9KTtcbn1cblxuRGlzcGF0Y2gucHJvdG90eXBlID0gZGlzcGF0Y2gucHJvdG90eXBlID0ge1xuICBjb25zdHJ1Y3RvcjogRGlzcGF0Y2gsXG4gIG9uOiBmdW5jdGlvbih0eXBlbmFtZSwgY2FsbGJhY2spIHtcbiAgICB2YXIgXyA9IHRoaXMuXyxcbiAgICAgICAgVCA9IHBhcnNlVHlwZW5hbWVzKHR5cGVuYW1lICsgXCJcIiwgXyksXG4gICAgICAgIHQsXG4gICAgICAgIGkgPSAtMSxcbiAgICAgICAgbiA9IFQubGVuZ3RoO1xuXG4gICAgLy8gSWYgbm8gY2FsbGJhY2sgd2FzIHNwZWNpZmllZCwgcmV0dXJuIHRoZSBjYWxsYmFjayBvZiB0aGUgZ2l2ZW4gdHlwZSBhbmQgbmFtZS5cbiAgICBpZiAoYXJndW1lbnRzLmxlbmd0aCA8IDIpIHtcbiAgICAgIHdoaWxlICgrK2kgPCBuKSBpZiAoKHQgPSAodHlwZW5hbWUgPSBUW2ldKS50eXBlKSAmJiAodCA9IGdldChfW3RdLCB0eXBlbmFtZS5uYW1lKSkpIHJldHVybiB0O1xuICAgICAgcmV0dXJuO1xuICAgIH1cblxuICAgIC8vIElmIGEgdHlwZSB3YXMgc3BlY2lmaWVkLCBzZXQgdGhlIGNhbGxiYWNrIGZvciB0aGUgZ2l2ZW4gdHlwZSBhbmQgbmFtZS5cbiAgICAvLyBPdGhlcndpc2UsIGlmIGEgbnVsbCBjYWxsYmFjayB3YXMgc3BlY2lmaWVkLCByZW1vdmUgY2FsbGJhY2tzIG9mIHRoZSBnaXZlbiBuYW1lLlxuICAgIGlmIChjYWxsYmFjayAhPSBudWxsICYmIHR5cGVvZiBjYWxsYmFjayAhPT0gXCJmdW5jdGlvblwiKSB0aHJvdyBuZXcgRXJyb3IoXCJpbnZhbGlkIGNhbGxiYWNrOiBcIiArIGNhbGxiYWNrKTtcbiAgICB3aGlsZSAoKytpIDwgbikge1xuICAgICAgaWYgKHQgPSAodHlwZW5hbWUgPSBUW2ldKS50eXBlKSBfW3RdID0gc2V0KF9bdF0sIHR5cGVuYW1lLm5hbWUsIGNhbGxiYWNrKTtcbiAgICAgIGVsc2UgaWYgKGNhbGxiYWNrID09IG51bGwpIGZvciAodCBpbiBfKSBfW3RdID0gc2V0KF9bdF0sIHR5cGVuYW1lLm5hbWUsIG51bGwpO1xuICAgIH1cblxuICAgIHJldHVybiB0aGlzO1xuICB9LFxuICBjb3B5OiBmdW5jdGlvbigpIHtcbiAgICB2YXIgY29weSA9IHt9LCBfID0gdGhpcy5fO1xuICAgIGZvciAodmFyIHQgaW4gXykgY29weVt0XSA9IF9bdF0uc2xpY2UoKTtcbiAgICByZXR1cm4gbmV3IERpc3BhdGNoKGNvcHkpO1xuICB9LFxuICBjYWxsOiBmdW5jdGlvbih0eXBlLCB0aGF0KSB7XG4gICAgaWYgKChuID0gYXJndW1lbnRzLmxlbmd0aCAtIDIpID4gMCkgZm9yICh2YXIgYXJncyA9IG5ldyBBcnJheShuKSwgaSA9IDAsIG4sIHQ7IGkgPCBuOyArK2kpIGFyZ3NbaV0gPSBhcmd1bWVudHNbaSArIDJdO1xuICAgIGlmICghdGhpcy5fLmhhc093blByb3BlcnR5KHR5cGUpKSB0aHJvdyBuZXcgRXJyb3IoXCJ1bmtub3duIHR5cGU6IFwiICsgdHlwZSk7XG4gICAgZm9yICh0ID0gdGhpcy5fW3R5cGVdLCBpID0gMCwgbiA9IHQubGVuZ3RoOyBpIDwgbjsgKytpKSB0W2ldLnZhbHVlLmFwcGx5KHRoYXQsIGFyZ3MpO1xuICB9LFxuICBhcHBseTogZnVuY3Rpb24odHlwZSwgdGhhdCwgYXJncykge1xuICAgIGlmICghdGhpcy5fLmhhc093blByb3BlcnR5KHR5cGUpKSB0aHJvdyBuZXcgRXJyb3IoXCJ1bmtub3duIHR5cGU6IFwiICsgdHlwZSk7XG4gICAgZm9yICh2YXIgdCA9IHRoaXMuX1t0eXBlXSwgaSA9IDAsIG4gPSB0Lmxlbmd0aDsgaSA8IG47ICsraSkgdFtpXS52YWx1ZS5hcHBseSh0aGF0LCBhcmdzKTtcbiAgfVxufTtcblxuZnVuY3Rpb24gZ2V0KHR5cGUsIG5hbWUpIHtcbiAgZm9yICh2YXIgaSA9IDAsIG4gPSB0eXBlLmxlbmd0aCwgYzsgaSA8IG47ICsraSkge1xuICAgIGlmICgoYyA9IHR5cGVbaV0pLm5hbWUgPT09IG5hbWUpIHtcbiAgICAgIHJldHVybiBjLnZhbHVlO1xuICAgIH1cbiAgfVxufVxuXG5mdW5jdGlvbiBzZXQodHlwZSwgbmFtZSwgY2FsbGJhY2spIHtcbiAgZm9yICh2YXIgaSA9IDAsIG4gPSB0eXBlLmxlbmd0aDsgaSA8IG47ICsraSkge1xuICAgIGlmICh0eXBlW2ldLm5hbWUgPT09IG5hbWUpIHtcbiAgICAgIHR5cGVbaV0gPSBub29wLCB0eXBlID0gdHlwZS5zbGljZSgwLCBpKS5jb25jYXQodHlwZS5zbGljZShpICsgMSkpO1xuICAgICAgYnJlYWs7XG4gICAgfVxuICB9XG4gIGlmIChjYWxsYmFjayAhPSBudWxsKSB0eXBlLnB1c2goe25hbWU6IG5hbWUsIHZhbHVlOiBjYWxsYmFja30pO1xuICByZXR1cm4gdHlwZTtcbn1cblxuZXhwb3J0IGRlZmF1bHQgZGlzcGF0Y2g7XG4iLCAiZXhwb3J0IHZhciB4aHRtbCA9IFwiaHR0cDovL3d3dy53My5vcmcvMTk5OS94aHRtbFwiO1xuXG5leHBvcnQgZGVmYXVsdCB7XG4gIHN2ZzogXCJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2Z1wiLFxuICB4aHRtbDogeGh0bWwsXG4gIHhsaW5rOiBcImh0dHA6Ly93d3cudzMub3JnLzE5OTkveGxpbmtcIixcbiAgeG1sOiBcImh0dHA6Ly93d3cudzMub3JnL1hNTC8xOTk4L25hbWVzcGFjZVwiLFxuICB4bWxuczogXCJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3htbG5zL1wiXG59O1xuIiwgImltcG9ydCBuYW1lc3BhY2VzIGZyb20gXCIuL25hbWVzcGFjZXMuanNcIjtcblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24obmFtZSkge1xuICB2YXIgcHJlZml4ID0gbmFtZSArPSBcIlwiLCBpID0gcHJlZml4LmluZGV4T2YoXCI6XCIpO1xuICBpZiAoaSA+PSAwICYmIChwcmVmaXggPSBuYW1lLnNsaWNlKDAsIGkpKSAhPT0gXCJ4bWxuc1wiKSBuYW1lID0gbmFtZS5zbGljZShpICsgMSk7XG4gIHJldHVybiBuYW1lc3BhY2VzLmhhc093blByb3BlcnR5KHByZWZpeCkgPyB7c3BhY2U6IG5hbWVzcGFjZXNbcHJlZml4XSwgbG9jYWw6IG5hbWV9IDogbmFtZTsgLy8gZXNsaW50LWRpc2FibGUtbGluZSBuby1wcm90b3R5cGUtYnVpbHRpbnNcbn1cbiIsICJpbXBvcnQgbmFtZXNwYWNlIGZyb20gXCIuL25hbWVzcGFjZS5qc1wiO1xuaW1wb3J0IHt4aHRtbH0gZnJvbSBcIi4vbmFtZXNwYWNlcy5qc1wiO1xuXG5mdW5jdGlvbiBjcmVhdG9ySW5oZXJpdChuYW1lKSB7XG4gIHJldHVybiBmdW5jdGlvbigpIHtcbiAgICB2YXIgZG9jdW1lbnQgPSB0aGlzLm93bmVyRG9jdW1lbnQsXG4gICAgICAgIHVyaSA9IHRoaXMubmFtZXNwYWNlVVJJO1xuICAgIHJldHVybiB1cmkgPT09IHhodG1sICYmIGRvY3VtZW50LmRvY3VtZW50RWxlbWVudC5uYW1lc3BhY2VVUkkgPT09IHhodG1sXG4gICAgICAgID8gZG9jdW1lbnQuY3JlYXRlRWxlbWVudChuYW1lKVxuICAgICAgICA6IGRvY3VtZW50LmNyZWF0ZUVsZW1lbnROUyh1cmksIG5hbWUpO1xuICB9O1xufVxuXG5mdW5jdGlvbiBjcmVhdG9yRml4ZWQoZnVsbG5hbWUpIHtcbiAgcmV0dXJuIGZ1bmN0aW9uKCkge1xuICAgIHJldHVybiB0aGlzLm93bmVyRG9jdW1lbnQuY3JlYXRlRWxlbWVudE5TKGZ1bGxuYW1lLnNwYWNlLCBmdWxsbmFtZS5sb2NhbCk7XG4gIH07XG59XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKG5hbWUpIHtcbiAgdmFyIGZ1bGxuYW1lID0gbmFtZXNwYWNlKG5hbWUpO1xuICByZXR1cm4gKGZ1bGxuYW1lLmxvY2FsXG4gICAgICA/IGNyZWF0b3JGaXhlZFxuICAgICAgOiBjcmVhdG9ySW5oZXJpdCkoZnVsbG5hbWUpO1xufVxuIiwgImZ1bmN0aW9uIG5vbmUoKSB7fVxuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbihzZWxlY3Rvcikge1xuICByZXR1cm4gc2VsZWN0b3IgPT0gbnVsbCA/IG5vbmUgOiBmdW5jdGlvbigpIHtcbiAgICByZXR1cm4gdGhpcy5xdWVyeVNlbGVjdG9yKHNlbGVjdG9yKTtcbiAgfTtcbn1cbiIsICJpbXBvcnQge1NlbGVjdGlvbn0gZnJvbSBcIi4vaW5kZXguanNcIjtcbmltcG9ydCBzZWxlY3RvciBmcm9tIFwiLi4vc2VsZWN0b3IuanNcIjtcblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24oc2VsZWN0KSB7XG4gIGlmICh0eXBlb2Ygc2VsZWN0ICE9PSBcImZ1bmN0aW9uXCIpIHNlbGVjdCA9IHNlbGVjdG9yKHNlbGVjdCk7XG5cbiAgZm9yICh2YXIgZ3JvdXBzID0gdGhpcy5fZ3JvdXBzLCBtID0gZ3JvdXBzLmxlbmd0aCwgc3ViZ3JvdXBzID0gbmV3IEFycmF5KG0pLCBqID0gMDsgaiA8IG07ICsraikge1xuICAgIGZvciAodmFyIGdyb3VwID0gZ3JvdXBzW2pdLCBuID0gZ3JvdXAubGVuZ3RoLCBzdWJncm91cCA9IHN1Ymdyb3Vwc1tqXSA9IG5ldyBBcnJheShuKSwgbm9kZSwgc3Vibm9kZSwgaSA9IDA7IGkgPCBuOyArK2kpIHtcbiAgICAgIGlmICgobm9kZSA9IGdyb3VwW2ldKSAmJiAoc3Vibm9kZSA9IHNlbGVjdC5jYWxsKG5vZGUsIG5vZGUuX19kYXRhX18sIGksIGdyb3VwKSkpIHtcbiAgICAgICAgaWYgKFwiX19kYXRhX19cIiBpbiBub2RlKSBzdWJub2RlLl9fZGF0YV9fID0gbm9kZS5fX2RhdGFfXztcbiAgICAgICAgc3ViZ3JvdXBbaV0gPSBzdWJub2RlO1xuICAgICAgfVxuICAgIH1cbiAgfVxuXG4gIHJldHVybiBuZXcgU2VsZWN0aW9uKHN1Ymdyb3VwcywgdGhpcy5fcGFyZW50cyk7XG59XG4iLCAiLy8gR2l2ZW4gc29tZXRoaW5nIGFycmF5IGxpa2UgKG9yIG51bGwpLCByZXR1cm5zIHNvbWV0aGluZyB0aGF0IGlzIHN0cmljdGx5IGFuXG4vLyBhcnJheS4gVGhpcyBpcyB1c2VkIHRvIGVuc3VyZSB0aGF0IGFycmF5LWxpa2Ugb2JqZWN0cyBwYXNzZWQgdG8gZDMuc2VsZWN0QWxsXG4vLyBvciBzZWxlY3Rpb24uc2VsZWN0QWxsIGFyZSBjb252ZXJ0ZWQgaW50byBwcm9wZXIgYXJyYXlzIHdoZW4gY3JlYXRpbmcgYVxuLy8gc2VsZWN0aW9uOyB3ZSBkb25cdTIwMTl0IGV2ZXIgd2FudCB0byBjcmVhdGUgYSBzZWxlY3Rpb24gYmFja2VkIGJ5IGEgbGl2ZVxuLy8gSFRNTENvbGxlY3Rpb24gb3IgTm9kZUxpc3QuIEhvd2V2ZXIsIG5vdGUgdGhhdCBzZWxlY3Rpb24uc2VsZWN0QWxsIHdpbGwgdXNlIGFcbi8vIHN0YXRpYyBOb2RlTGlzdCBhcyBhIGdyb3VwLCBzaW5jZSBpdCBzYWZlbHkgZGVyaXZlZCBmcm9tIHF1ZXJ5U2VsZWN0b3JBbGwuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbiBhcnJheSh4KSB7XG4gIHJldHVybiB4ID09IG51bGwgPyBbXSA6IEFycmF5LmlzQXJyYXkoeCkgPyB4IDogQXJyYXkuZnJvbSh4KTtcbn1cbiIsICJmdW5jdGlvbiBlbXB0eSgpIHtcbiAgcmV0dXJuIFtdO1xufVxuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbihzZWxlY3Rvcikge1xuICByZXR1cm4gc2VsZWN0b3IgPT0gbnVsbCA/IGVtcHR5IDogZnVuY3Rpb24oKSB7XG4gICAgcmV0dXJuIHRoaXMucXVlcnlTZWxlY3RvckFsbChzZWxlY3Rvcik7XG4gIH07XG59XG4iLCAiaW1wb3J0IHtTZWxlY3Rpb259IGZyb20gXCIuL2luZGV4LmpzXCI7XG5pbXBvcnQgYXJyYXkgZnJvbSBcIi4uL2FycmF5LmpzXCI7XG5pbXBvcnQgc2VsZWN0b3JBbGwgZnJvbSBcIi4uL3NlbGVjdG9yQWxsLmpzXCI7XG5cbmZ1bmN0aW9uIGFycmF5QWxsKHNlbGVjdCkge1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgcmV0dXJuIGFycmF5KHNlbGVjdC5hcHBseSh0aGlzLCBhcmd1bWVudHMpKTtcbiAgfTtcbn1cblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24oc2VsZWN0KSB7XG4gIGlmICh0eXBlb2Ygc2VsZWN0ID09PSBcImZ1bmN0aW9uXCIpIHNlbGVjdCA9IGFycmF5QWxsKHNlbGVjdCk7XG4gIGVsc2Ugc2VsZWN0ID0gc2VsZWN0b3JBbGwoc2VsZWN0KTtcblxuICBmb3IgKHZhciBncm91cHMgPSB0aGlzLl9ncm91cHMsIG0gPSBncm91cHMubGVuZ3RoLCBzdWJncm91cHMgPSBbXSwgcGFyZW50cyA9IFtdLCBqID0gMDsgaiA8IG07ICsraikge1xuICAgIGZvciAodmFyIGdyb3VwID0gZ3JvdXBzW2pdLCBuID0gZ3JvdXAubGVuZ3RoLCBub2RlLCBpID0gMDsgaSA8IG47ICsraSkge1xuICAgICAgaWYgKG5vZGUgPSBncm91cFtpXSkge1xuICAgICAgICBzdWJncm91cHMucHVzaChzZWxlY3QuY2FsbChub2RlLCBub2RlLl9fZGF0YV9fLCBpLCBncm91cCkpO1xuICAgICAgICBwYXJlbnRzLnB1c2gobm9kZSk7XG4gICAgICB9XG4gICAgfVxuICB9XG5cbiAgcmV0dXJuIG5ldyBTZWxlY3Rpb24oc3ViZ3JvdXBzLCBwYXJlbnRzKTtcbn1cbiIsICJleHBvcnQgZGVmYXVsdCBmdW5jdGlvbihzZWxlY3Rvcikge1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgcmV0dXJuIHRoaXMubWF0Y2hlcyhzZWxlY3Rvcik7XG4gIH07XG59XG5cbmV4cG9ydCBmdW5jdGlvbiBjaGlsZE1hdGNoZXIoc2VsZWN0b3IpIHtcbiAgcmV0dXJuIGZ1bmN0aW9uKG5vZGUpIHtcbiAgICByZXR1cm4gbm9kZS5tYXRjaGVzKHNlbGVjdG9yKTtcbiAgfTtcbn1cblxuIiwgImltcG9ydCB7Y2hpbGRNYXRjaGVyfSBmcm9tIFwiLi4vbWF0Y2hlci5qc1wiO1xuXG52YXIgZmluZCA9IEFycmF5LnByb3RvdHlwZS5maW5kO1xuXG5mdW5jdGlvbiBjaGlsZEZpbmQobWF0Y2gpIHtcbiAgcmV0dXJuIGZ1bmN0aW9uKCkge1xuICAgIHJldHVybiBmaW5kLmNhbGwodGhpcy5jaGlsZHJlbiwgbWF0Y2gpO1xuICB9O1xufVxuXG5mdW5jdGlvbiBjaGlsZEZpcnN0KCkge1xuICByZXR1cm4gdGhpcy5maXJzdEVsZW1lbnRDaGlsZDtcbn1cblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24obWF0Y2gpIHtcbiAgcmV0dXJuIHRoaXMuc2VsZWN0KG1hdGNoID09IG51bGwgPyBjaGlsZEZpcnN0XG4gICAgICA6IGNoaWxkRmluZCh0eXBlb2YgbWF0Y2ggPT09IFwiZnVuY3Rpb25cIiA/IG1hdGNoIDogY2hpbGRNYXRjaGVyKG1hdGNoKSkpO1xufVxuIiwgImltcG9ydCB7Y2hpbGRNYXRjaGVyfSBmcm9tIFwiLi4vbWF0Y2hlci5qc1wiO1xuXG52YXIgZmlsdGVyID0gQXJyYXkucHJvdG90eXBlLmZpbHRlcjtcblxuZnVuY3Rpb24gY2hpbGRyZW4oKSB7XG4gIHJldHVybiBBcnJheS5mcm9tKHRoaXMuY2hpbGRyZW4pO1xufVxuXG5mdW5jdGlvbiBjaGlsZHJlbkZpbHRlcihtYXRjaCkge1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgcmV0dXJuIGZpbHRlci5jYWxsKHRoaXMuY2hpbGRyZW4sIG1hdGNoKTtcbiAgfTtcbn1cblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24obWF0Y2gpIHtcbiAgcmV0dXJuIHRoaXMuc2VsZWN0QWxsKG1hdGNoID09IG51bGwgPyBjaGlsZHJlblxuICAgICAgOiBjaGlsZHJlbkZpbHRlcih0eXBlb2YgbWF0Y2ggPT09IFwiZnVuY3Rpb25cIiA/IG1hdGNoIDogY2hpbGRNYXRjaGVyKG1hdGNoKSkpO1xufVxuIiwgImltcG9ydCB7U2VsZWN0aW9ufSBmcm9tIFwiLi9pbmRleC5qc1wiO1xuaW1wb3J0IG1hdGNoZXIgZnJvbSBcIi4uL21hdGNoZXIuanNcIjtcblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24obWF0Y2gpIHtcbiAgaWYgKHR5cGVvZiBtYXRjaCAhPT0gXCJmdW5jdGlvblwiKSBtYXRjaCA9IG1hdGNoZXIobWF0Y2gpO1xuXG4gIGZvciAodmFyIGdyb3VwcyA9IHRoaXMuX2dyb3VwcywgbSA9IGdyb3Vwcy5sZW5ndGgsIHN1Ymdyb3VwcyA9IG5ldyBBcnJheShtKSwgaiA9IDA7IGogPCBtOyArK2opIHtcbiAgICBmb3IgKHZhciBncm91cCA9IGdyb3Vwc1tqXSwgbiA9IGdyb3VwLmxlbmd0aCwgc3ViZ3JvdXAgPSBzdWJncm91cHNbal0gPSBbXSwgbm9kZSwgaSA9IDA7IGkgPCBuOyArK2kpIHtcbiAgICAgIGlmICgobm9kZSA9IGdyb3VwW2ldKSAmJiBtYXRjaC5jYWxsKG5vZGUsIG5vZGUuX19kYXRhX18sIGksIGdyb3VwKSkge1xuICAgICAgICBzdWJncm91cC5wdXNoKG5vZGUpO1xuICAgICAgfVxuICAgIH1cbiAgfVxuXG4gIHJldHVybiBuZXcgU2VsZWN0aW9uKHN1Ymdyb3VwcywgdGhpcy5fcGFyZW50cyk7XG59XG4iLCAiZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24odXBkYXRlKSB7XG4gIHJldHVybiBuZXcgQXJyYXkodXBkYXRlLmxlbmd0aCk7XG59XG4iLCAiaW1wb3J0IHNwYXJzZSBmcm9tIFwiLi9zcGFyc2UuanNcIjtcbmltcG9ydCB7U2VsZWN0aW9ufSBmcm9tIFwiLi9pbmRleC5qc1wiO1xuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbigpIHtcbiAgcmV0dXJuIG5ldyBTZWxlY3Rpb24odGhpcy5fZW50ZXIgfHwgdGhpcy5fZ3JvdXBzLm1hcChzcGFyc2UpLCB0aGlzLl9wYXJlbnRzKTtcbn1cblxuZXhwb3J0IGZ1bmN0aW9uIEVudGVyTm9kZShwYXJlbnQsIGRhdHVtKSB7XG4gIHRoaXMub3duZXJEb2N1bWVudCA9IHBhcmVudC5vd25lckRvY3VtZW50O1xuICB0aGlzLm5hbWVzcGFjZVVSSSA9IHBhcmVudC5uYW1lc3BhY2VVUkk7XG4gIHRoaXMuX25leHQgPSBudWxsO1xuICB0aGlzLl9wYXJlbnQgPSBwYXJlbnQ7XG4gIHRoaXMuX19kYXRhX18gPSBkYXR1bTtcbn1cblxuRW50ZXJOb2RlLnByb3RvdHlwZSA9IHtcbiAgY29uc3RydWN0b3I6IEVudGVyTm9kZSxcbiAgYXBwZW5kQ2hpbGQ6IGZ1bmN0aW9uKGNoaWxkKSB7IHJldHVybiB0aGlzLl9wYXJlbnQuaW5zZXJ0QmVmb3JlKGNoaWxkLCB0aGlzLl9uZXh0KTsgfSxcbiAgaW5zZXJ0QmVmb3JlOiBmdW5jdGlvbihjaGlsZCwgbmV4dCkgeyByZXR1cm4gdGhpcy5fcGFyZW50Lmluc2VydEJlZm9yZShjaGlsZCwgbmV4dCk7IH0sXG4gIHF1ZXJ5U2VsZWN0b3I6IGZ1bmN0aW9uKHNlbGVjdG9yKSB7IHJldHVybiB0aGlzLl9wYXJlbnQucXVlcnlTZWxlY3RvcihzZWxlY3Rvcik7IH0sXG4gIHF1ZXJ5U2VsZWN0b3JBbGw6IGZ1bmN0aW9uKHNlbGVjdG9yKSB7IHJldHVybiB0aGlzLl9wYXJlbnQucXVlcnlTZWxlY3RvckFsbChzZWxlY3Rvcik7IH1cbn07XG4iLCAiZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24oeCkge1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgcmV0dXJuIHg7XG4gIH07XG59XG4iLCAiaW1wb3J0IHtTZWxlY3Rpb259IGZyb20gXCIuL2luZGV4LmpzXCI7XG5pbXBvcnQge0VudGVyTm9kZX0gZnJvbSBcIi4vZW50ZXIuanNcIjtcbmltcG9ydCBjb25zdGFudCBmcm9tIFwiLi4vY29uc3RhbnQuanNcIjtcblxuZnVuY3Rpb24gYmluZEluZGV4KHBhcmVudCwgZ3JvdXAsIGVudGVyLCB1cGRhdGUsIGV4aXQsIGRhdGEpIHtcbiAgdmFyIGkgPSAwLFxuICAgICAgbm9kZSxcbiAgICAgIGdyb3VwTGVuZ3RoID0gZ3JvdXAubGVuZ3RoLFxuICAgICAgZGF0YUxlbmd0aCA9IGRhdGEubGVuZ3RoO1xuXG4gIC8vIFB1dCBhbnkgbm9uLW51bGwgbm9kZXMgdGhhdCBmaXQgaW50byB1cGRhdGUuXG4gIC8vIFB1dCBhbnkgbnVsbCBub2RlcyBpbnRvIGVudGVyLlxuICAvLyBQdXQgYW55IHJlbWFpbmluZyBkYXRhIGludG8gZW50ZXIuXG4gIGZvciAoOyBpIDwgZGF0YUxlbmd0aDsgKytpKSB7XG4gICAgaWYgKG5vZGUgPSBncm91cFtpXSkge1xuICAgICAgbm9kZS5fX2RhdGFfXyA9IGRhdGFbaV07XG4gICAgICB1cGRhdGVbaV0gPSBub2RlO1xuICAgIH0gZWxzZSB7XG4gICAgICBlbnRlcltpXSA9IG5ldyBFbnRlck5vZGUocGFyZW50LCBkYXRhW2ldKTtcbiAgICB9XG4gIH1cblxuICAvLyBQdXQgYW55IG5vbi1udWxsIG5vZGVzIHRoYXQgZG9uXHUyMDE5dCBmaXQgaW50byBleGl0LlxuICBmb3IgKDsgaSA8IGdyb3VwTGVuZ3RoOyArK2kpIHtcbiAgICBpZiAobm9kZSA9IGdyb3VwW2ldKSB7XG4gICAgICBleGl0W2ldID0gbm9kZTtcbiAgICB9XG4gIH1cbn1cblxuZnVuY3Rpb24gYmluZEtleShwYXJlbnQsIGdyb3VwLCBlbnRlciwgdXBkYXRlLCBleGl0LCBkYXRhLCBrZXkpIHtcbiAgdmFyIGksXG4gICAgICBub2RlLFxuICAgICAgbm9kZUJ5S2V5VmFsdWUgPSBuZXcgTWFwLFxuICAgICAgZ3JvdXBMZW5ndGggPSBncm91cC5sZW5ndGgsXG4gICAgICBkYXRhTGVuZ3RoID0gZGF0YS5sZW5ndGgsXG4gICAgICBrZXlWYWx1ZXMgPSBuZXcgQXJyYXkoZ3JvdXBMZW5ndGgpLFxuICAgICAga2V5VmFsdWU7XG5cbiAgLy8gQ29tcHV0ZSB0aGUga2V5IGZvciBlYWNoIG5vZGUuXG4gIC8vIElmIG11bHRpcGxlIG5vZGVzIGhhdmUgdGhlIHNhbWUga2V5LCB0aGUgZHVwbGljYXRlcyBhcmUgYWRkZWQgdG8gZXhpdC5cbiAgZm9yIChpID0gMDsgaSA8IGdyb3VwTGVuZ3RoOyArK2kpIHtcbiAgICBpZiAobm9kZSA9IGdyb3VwW2ldKSB7XG4gICAgICBrZXlWYWx1ZXNbaV0gPSBrZXlWYWx1ZSA9IGtleS5jYWxsKG5vZGUsIG5vZGUuX19kYXRhX18sIGksIGdyb3VwKSArIFwiXCI7XG4gICAgICBpZiAobm9kZUJ5S2V5VmFsdWUuaGFzKGtleVZhbHVlKSkge1xuICAgICAgICBleGl0W2ldID0gbm9kZTtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIG5vZGVCeUtleVZhbHVlLnNldChrZXlWYWx1ZSwgbm9kZSk7XG4gICAgICB9XG4gICAgfVxuICB9XG5cbiAgLy8gQ29tcHV0ZSB0aGUga2V5IGZvciBlYWNoIGRhdHVtLlxuICAvLyBJZiB0aGVyZSBhIG5vZGUgYXNzb2NpYXRlZCB3aXRoIHRoaXMga2V5LCBqb2luIGFuZCBhZGQgaXQgdG8gdXBkYXRlLlxuICAvLyBJZiB0aGVyZSBpcyBub3QgKG9yIHRoZSBrZXkgaXMgYSBkdXBsaWNhdGUpLCBhZGQgaXQgdG8gZW50ZXIuXG4gIGZvciAoaSA9IDA7IGkgPCBkYXRhTGVuZ3RoOyArK2kpIHtcbiAgICBrZXlWYWx1ZSA9IGtleS5jYWxsKHBhcmVudCwgZGF0YVtpXSwgaSwgZGF0YSkgKyBcIlwiO1xuICAgIGlmIChub2RlID0gbm9kZUJ5S2V5VmFsdWUuZ2V0KGtleVZhbHVlKSkge1xuICAgICAgdXBkYXRlW2ldID0gbm9kZTtcbiAgICAgIG5vZGUuX19kYXRhX18gPSBkYXRhW2ldO1xuICAgICAgbm9kZUJ5S2V5VmFsdWUuZGVsZXRlKGtleVZhbHVlKTtcbiAgICB9IGVsc2Uge1xuICAgICAgZW50ZXJbaV0gPSBuZXcgRW50ZXJOb2RlKHBhcmVudCwgZGF0YVtpXSk7XG4gICAgfVxuICB9XG5cbiAgLy8gQWRkIGFueSByZW1haW5pbmcgbm9kZXMgdGhhdCB3ZXJlIG5vdCBib3VuZCB0byBkYXRhIHRvIGV4aXQuXG4gIGZvciAoaSA9IDA7IGkgPCBncm91cExlbmd0aDsgKytpKSB7XG4gICAgaWYgKChub2RlID0gZ3JvdXBbaV0pICYmIChub2RlQnlLZXlWYWx1ZS5nZXQoa2V5VmFsdWVzW2ldKSA9PT0gbm9kZSkpIHtcbiAgICAgIGV4aXRbaV0gPSBub2RlO1xuICAgIH1cbiAgfVxufVxuXG5mdW5jdGlvbiBkYXR1bShub2RlKSB7XG4gIHJldHVybiBub2RlLl9fZGF0YV9fO1xufVxuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbih2YWx1ZSwga2V5KSB7XG4gIGlmICghYXJndW1lbnRzLmxlbmd0aCkgcmV0dXJuIEFycmF5LmZyb20odGhpcywgZGF0dW0pO1xuXG4gIHZhciBiaW5kID0ga2V5ID8gYmluZEtleSA6IGJpbmRJbmRleCxcbiAgICAgIHBhcmVudHMgPSB0aGlzLl9wYXJlbnRzLFxuICAgICAgZ3JvdXBzID0gdGhpcy5fZ3JvdXBzO1xuXG4gIGlmICh0eXBlb2YgdmFsdWUgIT09IFwiZnVuY3Rpb25cIikgdmFsdWUgPSBjb25zdGFudCh2YWx1ZSk7XG5cbiAgZm9yICh2YXIgbSA9IGdyb3Vwcy5sZW5ndGgsIHVwZGF0ZSA9IG5ldyBBcnJheShtKSwgZW50ZXIgPSBuZXcgQXJyYXkobSksIGV4aXQgPSBuZXcgQXJyYXkobSksIGogPSAwOyBqIDwgbTsgKytqKSB7XG4gICAgdmFyIHBhcmVudCA9IHBhcmVudHNbal0sXG4gICAgICAgIGdyb3VwID0gZ3JvdXBzW2pdLFxuICAgICAgICBncm91cExlbmd0aCA9IGdyb3VwLmxlbmd0aCxcbiAgICAgICAgZGF0YSA9IGFycmF5bGlrZSh2YWx1ZS5jYWxsKHBhcmVudCwgcGFyZW50ICYmIHBhcmVudC5fX2RhdGFfXywgaiwgcGFyZW50cykpLFxuICAgICAgICBkYXRhTGVuZ3RoID0gZGF0YS5sZW5ndGgsXG4gICAgICAgIGVudGVyR3JvdXAgPSBlbnRlcltqXSA9IG5ldyBBcnJheShkYXRhTGVuZ3RoKSxcbiAgICAgICAgdXBkYXRlR3JvdXAgPSB1cGRhdGVbal0gPSBuZXcgQXJyYXkoZGF0YUxlbmd0aCksXG4gICAgICAgIGV4aXRHcm91cCA9IGV4aXRbal0gPSBuZXcgQXJyYXkoZ3JvdXBMZW5ndGgpO1xuXG4gICAgYmluZChwYXJlbnQsIGdyb3VwLCBlbnRlckdyb3VwLCB1cGRhdGVHcm91cCwgZXhpdEdyb3VwLCBkYXRhLCBrZXkpO1xuXG4gICAgLy8gTm93IGNvbm5lY3QgdGhlIGVudGVyIG5vZGVzIHRvIHRoZWlyIGZvbGxvd2luZyB1cGRhdGUgbm9kZSwgc3VjaCB0aGF0XG4gICAgLy8gYXBwZW5kQ2hpbGQgY2FuIGluc2VydCB0aGUgbWF0ZXJpYWxpemVkIGVudGVyIG5vZGUgYmVmb3JlIHRoaXMgbm9kZSxcbiAgICAvLyByYXRoZXIgdGhhbiBhdCB0aGUgZW5kIG9mIHRoZSBwYXJlbnQgbm9kZS5cbiAgICBmb3IgKHZhciBpMCA9IDAsIGkxID0gMCwgcHJldmlvdXMsIG5leHQ7IGkwIDwgZGF0YUxlbmd0aDsgKytpMCkge1xuICAgICAgaWYgKHByZXZpb3VzID0gZW50ZXJHcm91cFtpMF0pIHtcbiAgICAgICAgaWYgKGkwID49IGkxKSBpMSA9IGkwICsgMTtcbiAgICAgICAgd2hpbGUgKCEobmV4dCA9IHVwZGF0ZUdyb3VwW2kxXSkgJiYgKytpMSA8IGRhdGFMZW5ndGgpO1xuICAgICAgICBwcmV2aW91cy5fbmV4dCA9IG5leHQgfHwgbnVsbDtcbiAgICAgIH1cbiAgICB9XG4gIH1cblxuICB1cGRhdGUgPSBuZXcgU2VsZWN0aW9uKHVwZGF0ZSwgcGFyZW50cyk7XG4gIHVwZGF0ZS5fZW50ZXIgPSBlbnRlcjtcbiAgdXBkYXRlLl9leGl0ID0gZXhpdDtcbiAgcmV0dXJuIHVwZGF0ZTtcbn1cblxuLy8gR2l2ZW4gc29tZSBkYXRhLCB0aGlzIHJldHVybnMgYW4gYXJyYXktbGlrZSB2aWV3IG9mIGl0OiBhbiBvYmplY3QgdGhhdFxuLy8gZXhwb3NlcyBhIGxlbmd0aCBwcm9wZXJ0eSBhbmQgYWxsb3dzIG51bWVyaWMgaW5kZXhpbmcuIE5vdGUgdGhhdCB1bmxpa2Vcbi8vIHNlbGVjdEFsbCwgdGhpcyBpc25cdTIwMTl0IHdvcnJpZWQgYWJvdXQgXHUyMDFDbGl2ZVx1MjAxRCBjb2xsZWN0aW9ucyBiZWNhdXNlIHRoZSByZXN1bHRpbmdcbi8vIGFycmF5IHdpbGwgb25seSBiZSB1c2VkIGJyaWVmbHkgd2hpbGUgZGF0YSBpcyBiZWluZyBib3VuZC4gKEl0IGlzIHBvc3NpYmxlIHRvXG4vLyBjYXVzZSB0aGUgZGF0YSB0byBjaGFuZ2Ugd2hpbGUgaXRlcmF0aW5nIGJ5IHVzaW5nIGEga2V5IGZ1bmN0aW9uLCBidXQgcGxlYXNlXG4vLyBkb25cdTIwMTl0OyB3ZVx1MjAxOWQgcmF0aGVyIGF2b2lkIGEgZ3JhdHVpdG91cyBjb3B5LilcbmZ1bmN0aW9uIGFycmF5bGlrZShkYXRhKSB7XG4gIHJldHVybiB0eXBlb2YgZGF0YSA9PT0gXCJvYmplY3RcIiAmJiBcImxlbmd0aFwiIGluIGRhdGFcbiAgICA/IGRhdGEgLy8gQXJyYXksIFR5cGVkQXJyYXksIE5vZGVMaXN0LCBhcnJheS1saWtlXG4gICAgOiBBcnJheS5mcm9tKGRhdGEpOyAvLyBNYXAsIFNldCwgaXRlcmFibGUsIHN0cmluZywgb3IgYW55dGhpbmcgZWxzZVxufVxuIiwgImltcG9ydCBzcGFyc2UgZnJvbSBcIi4vc3BhcnNlLmpzXCI7XG5pbXBvcnQge1NlbGVjdGlvbn0gZnJvbSBcIi4vaW5kZXguanNcIjtcblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24oKSB7XG4gIHJldHVybiBuZXcgU2VsZWN0aW9uKHRoaXMuX2V4aXQgfHwgdGhpcy5fZ3JvdXBzLm1hcChzcGFyc2UpLCB0aGlzLl9wYXJlbnRzKTtcbn1cbiIsICJleHBvcnQgZGVmYXVsdCBmdW5jdGlvbihvbmVudGVyLCBvbnVwZGF0ZSwgb25leGl0KSB7XG4gIHZhciBlbnRlciA9IHRoaXMuZW50ZXIoKSwgdXBkYXRlID0gdGhpcywgZXhpdCA9IHRoaXMuZXhpdCgpO1xuICBpZiAodHlwZW9mIG9uZW50ZXIgPT09IFwiZnVuY3Rpb25cIikge1xuICAgIGVudGVyID0gb25lbnRlcihlbnRlcik7XG4gICAgaWYgKGVudGVyKSBlbnRlciA9IGVudGVyLnNlbGVjdGlvbigpO1xuICB9IGVsc2Uge1xuICAgIGVudGVyID0gZW50ZXIuYXBwZW5kKG9uZW50ZXIgKyBcIlwiKTtcbiAgfVxuICBpZiAob251cGRhdGUgIT0gbnVsbCkge1xuICAgIHVwZGF0ZSA9IG9udXBkYXRlKHVwZGF0ZSk7XG4gICAgaWYgKHVwZGF0ZSkgdXBkYXRlID0gdXBkYXRlLnNlbGVjdGlvbigpO1xuICB9XG4gIGlmIChvbmV4aXQgPT0gbnVsbCkgZXhpdC5yZW1vdmUoKTsgZWxzZSBvbmV4aXQoZXhpdCk7XG4gIHJldHVybiBlbnRlciAmJiB1cGRhdGUgPyBlbnRlci5tZXJnZSh1cGRhdGUpLm9yZGVyKCkgOiB1cGRhdGU7XG59XG4iLCAiaW1wb3J0IHtTZWxlY3Rpb259IGZyb20gXCIuL2luZGV4LmpzXCI7XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKGNvbnRleHQpIHtcbiAgdmFyIHNlbGVjdGlvbiA9IGNvbnRleHQuc2VsZWN0aW9uID8gY29udGV4dC5zZWxlY3Rpb24oKSA6IGNvbnRleHQ7XG5cbiAgZm9yICh2YXIgZ3JvdXBzMCA9IHRoaXMuX2dyb3VwcywgZ3JvdXBzMSA9IHNlbGVjdGlvbi5fZ3JvdXBzLCBtMCA9IGdyb3VwczAubGVuZ3RoLCBtMSA9IGdyb3VwczEubGVuZ3RoLCBtID0gTWF0aC5taW4obTAsIG0xKSwgbWVyZ2VzID0gbmV3IEFycmF5KG0wKSwgaiA9IDA7IGogPCBtOyArK2opIHtcbiAgICBmb3IgKHZhciBncm91cDAgPSBncm91cHMwW2pdLCBncm91cDEgPSBncm91cHMxW2pdLCBuID0gZ3JvdXAwLmxlbmd0aCwgbWVyZ2UgPSBtZXJnZXNbal0gPSBuZXcgQXJyYXkobiksIG5vZGUsIGkgPSAwOyBpIDwgbjsgKytpKSB7XG4gICAgICBpZiAobm9kZSA9IGdyb3VwMFtpXSB8fCBncm91cDFbaV0pIHtcbiAgICAgICAgbWVyZ2VbaV0gPSBub2RlO1xuICAgICAgfVxuICAgIH1cbiAgfVxuXG4gIGZvciAoOyBqIDwgbTA7ICsraikge1xuICAgIG1lcmdlc1tqXSA9IGdyb3VwczBbal07XG4gIH1cblxuICByZXR1cm4gbmV3IFNlbGVjdGlvbihtZXJnZXMsIHRoaXMuX3BhcmVudHMpO1xufVxuIiwgImV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKCkge1xuXG4gIGZvciAodmFyIGdyb3VwcyA9IHRoaXMuX2dyb3VwcywgaiA9IC0xLCBtID0gZ3JvdXBzLmxlbmd0aDsgKytqIDwgbTspIHtcbiAgICBmb3IgKHZhciBncm91cCA9IGdyb3Vwc1tqXSwgaSA9IGdyb3VwLmxlbmd0aCAtIDEsIG5leHQgPSBncm91cFtpXSwgbm9kZTsgLS1pID49IDA7KSB7XG4gICAgICBpZiAobm9kZSA9IGdyb3VwW2ldKSB7XG4gICAgICAgIGlmIChuZXh0ICYmIG5vZGUuY29tcGFyZURvY3VtZW50UG9zaXRpb24obmV4dCkgXiA0KSBuZXh0LnBhcmVudE5vZGUuaW5zZXJ0QmVmb3JlKG5vZGUsIG5leHQpO1xuICAgICAgICBuZXh0ID0gbm9kZTtcbiAgICAgIH1cbiAgICB9XG4gIH1cblxuICByZXR1cm4gdGhpcztcbn1cbiIsICJpbXBvcnQge1NlbGVjdGlvbn0gZnJvbSBcIi4vaW5kZXguanNcIjtcblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24oY29tcGFyZSkge1xuICBpZiAoIWNvbXBhcmUpIGNvbXBhcmUgPSBhc2NlbmRpbmc7XG5cbiAgZnVuY3Rpb24gY29tcGFyZU5vZGUoYSwgYikge1xuICAgIHJldHVybiBhICYmIGIgPyBjb21wYXJlKGEuX19kYXRhX18sIGIuX19kYXRhX18pIDogIWEgLSAhYjtcbiAgfVxuXG4gIGZvciAodmFyIGdyb3VwcyA9IHRoaXMuX2dyb3VwcywgbSA9IGdyb3Vwcy5sZW5ndGgsIHNvcnRncm91cHMgPSBuZXcgQXJyYXkobSksIGogPSAwOyBqIDwgbTsgKytqKSB7XG4gICAgZm9yICh2YXIgZ3JvdXAgPSBncm91cHNbal0sIG4gPSBncm91cC5sZW5ndGgsIHNvcnRncm91cCA9IHNvcnRncm91cHNbal0gPSBuZXcgQXJyYXkobiksIG5vZGUsIGkgPSAwOyBpIDwgbjsgKytpKSB7XG4gICAgICBpZiAobm9kZSA9IGdyb3VwW2ldKSB7XG4gICAgICAgIHNvcnRncm91cFtpXSA9IG5vZGU7XG4gICAgICB9XG4gICAgfVxuICAgIHNvcnRncm91cC5zb3J0KGNvbXBhcmVOb2RlKTtcbiAgfVxuXG4gIHJldHVybiBuZXcgU2VsZWN0aW9uKHNvcnRncm91cHMsIHRoaXMuX3BhcmVudHMpLm9yZGVyKCk7XG59XG5cbmZ1bmN0aW9uIGFzY2VuZGluZyhhLCBiKSB7XG4gIHJldHVybiBhIDwgYiA/IC0xIDogYSA+IGIgPyAxIDogYSA+PSBiID8gMCA6IE5hTjtcbn1cbiIsICJleHBvcnQgZGVmYXVsdCBmdW5jdGlvbigpIHtcbiAgdmFyIGNhbGxiYWNrID0gYXJndW1lbnRzWzBdO1xuICBhcmd1bWVudHNbMF0gPSB0aGlzO1xuICBjYWxsYmFjay5hcHBseShudWxsLCBhcmd1bWVudHMpO1xuICByZXR1cm4gdGhpcztcbn1cbiIsICJleHBvcnQgZGVmYXVsdCBmdW5jdGlvbigpIHtcbiAgcmV0dXJuIEFycmF5LmZyb20odGhpcyk7XG59XG4iLCAiZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24oKSB7XG5cbiAgZm9yICh2YXIgZ3JvdXBzID0gdGhpcy5fZ3JvdXBzLCBqID0gMCwgbSA9IGdyb3Vwcy5sZW5ndGg7IGogPCBtOyArK2opIHtcbiAgICBmb3IgKHZhciBncm91cCA9IGdyb3Vwc1tqXSwgaSA9IDAsIG4gPSBncm91cC5sZW5ndGg7IGkgPCBuOyArK2kpIHtcbiAgICAgIHZhciBub2RlID0gZ3JvdXBbaV07XG4gICAgICBpZiAobm9kZSkgcmV0dXJuIG5vZGU7XG4gICAgfVxuICB9XG5cbiAgcmV0dXJuIG51bGw7XG59XG4iLCAiZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24oKSB7XG4gIGxldCBzaXplID0gMDtcbiAgZm9yIChjb25zdCBub2RlIG9mIHRoaXMpICsrc2l6ZTsgLy8gZXNsaW50LWRpc2FibGUtbGluZSBuby11bnVzZWQtdmFyc1xuICByZXR1cm4gc2l6ZTtcbn1cbiIsICJleHBvcnQgZGVmYXVsdCBmdW5jdGlvbigpIHtcbiAgcmV0dXJuICF0aGlzLm5vZGUoKTtcbn1cbiIsICJleHBvcnQgZGVmYXVsdCBmdW5jdGlvbihjYWxsYmFjaykge1xuXG4gIGZvciAodmFyIGdyb3VwcyA9IHRoaXMuX2dyb3VwcywgaiA9IDAsIG0gPSBncm91cHMubGVuZ3RoOyBqIDwgbTsgKytqKSB7XG4gICAgZm9yICh2YXIgZ3JvdXAgPSBncm91cHNbal0sIGkgPSAwLCBuID0gZ3JvdXAubGVuZ3RoLCBub2RlOyBpIDwgbjsgKytpKSB7XG4gICAgICBpZiAobm9kZSA9IGdyb3VwW2ldKSBjYWxsYmFjay5jYWxsKG5vZGUsIG5vZGUuX19kYXRhX18sIGksIGdyb3VwKTtcbiAgICB9XG4gIH1cblxuICByZXR1cm4gdGhpcztcbn1cbiIsICJpbXBvcnQgbmFtZXNwYWNlIGZyb20gXCIuLi9uYW1lc3BhY2UuanNcIjtcblxuZnVuY3Rpb24gYXR0clJlbW92ZShuYW1lKSB7XG4gIHJldHVybiBmdW5jdGlvbigpIHtcbiAgICB0aGlzLnJlbW92ZUF0dHJpYnV0ZShuYW1lKTtcbiAgfTtcbn1cblxuZnVuY3Rpb24gYXR0clJlbW92ZU5TKGZ1bGxuYW1lKSB7XG4gIHJldHVybiBmdW5jdGlvbigpIHtcbiAgICB0aGlzLnJlbW92ZUF0dHJpYnV0ZU5TKGZ1bGxuYW1lLnNwYWNlLCBmdWxsbmFtZS5sb2NhbCk7XG4gIH07XG59XG5cbmZ1bmN0aW9uIGF0dHJDb25zdGFudChuYW1lLCB2YWx1ZSkge1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgdGhpcy5zZXRBdHRyaWJ1dGUobmFtZSwgdmFsdWUpO1xuICB9O1xufVxuXG5mdW5jdGlvbiBhdHRyQ29uc3RhbnROUyhmdWxsbmFtZSwgdmFsdWUpIHtcbiAgcmV0dXJuIGZ1bmN0aW9uKCkge1xuICAgIHRoaXMuc2V0QXR0cmlidXRlTlMoZnVsbG5hbWUuc3BhY2UsIGZ1bGxuYW1lLmxvY2FsLCB2YWx1ZSk7XG4gIH07XG59XG5cbmZ1bmN0aW9uIGF0dHJGdW5jdGlvbihuYW1lLCB2YWx1ZSkge1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgdmFyIHYgPSB2YWx1ZS5hcHBseSh0aGlzLCBhcmd1bWVudHMpO1xuICAgIGlmICh2ID09IG51bGwpIHRoaXMucmVtb3ZlQXR0cmlidXRlKG5hbWUpO1xuICAgIGVsc2UgdGhpcy5zZXRBdHRyaWJ1dGUobmFtZSwgdik7XG4gIH07XG59XG5cbmZ1bmN0aW9uIGF0dHJGdW5jdGlvbk5TKGZ1bGxuYW1lLCB2YWx1ZSkge1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgdmFyIHYgPSB2YWx1ZS5hcHBseSh0aGlzLCBhcmd1bWVudHMpO1xuICAgIGlmICh2ID09IG51bGwpIHRoaXMucmVtb3ZlQXR0cmlidXRlTlMoZnVsbG5hbWUuc3BhY2UsIGZ1bGxuYW1lLmxvY2FsKTtcbiAgICBlbHNlIHRoaXMuc2V0QXR0cmlidXRlTlMoZnVsbG5hbWUuc3BhY2UsIGZ1bGxuYW1lLmxvY2FsLCB2KTtcbiAgfTtcbn1cblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24obmFtZSwgdmFsdWUpIHtcbiAgdmFyIGZ1bGxuYW1lID0gbmFtZXNwYWNlKG5hbWUpO1xuXG4gIGlmIChhcmd1bWVudHMubGVuZ3RoIDwgMikge1xuICAgIHZhciBub2RlID0gdGhpcy5ub2RlKCk7XG4gICAgcmV0dXJuIGZ1bGxuYW1lLmxvY2FsXG4gICAgICAgID8gbm9kZS5nZXRBdHRyaWJ1dGVOUyhmdWxsbmFtZS5zcGFjZSwgZnVsbG5hbWUubG9jYWwpXG4gICAgICAgIDogbm9kZS5nZXRBdHRyaWJ1dGUoZnVsbG5hbWUpO1xuICB9XG5cbiAgcmV0dXJuIHRoaXMuZWFjaCgodmFsdWUgPT0gbnVsbFxuICAgICAgPyAoZnVsbG5hbWUubG9jYWwgPyBhdHRyUmVtb3ZlTlMgOiBhdHRyUmVtb3ZlKSA6ICh0eXBlb2YgdmFsdWUgPT09IFwiZnVuY3Rpb25cIlxuICAgICAgPyAoZnVsbG5hbWUubG9jYWwgPyBhdHRyRnVuY3Rpb25OUyA6IGF0dHJGdW5jdGlvbilcbiAgICAgIDogKGZ1bGxuYW1lLmxvY2FsID8gYXR0ckNvbnN0YW50TlMgOiBhdHRyQ29uc3RhbnQpKSkoZnVsbG5hbWUsIHZhbHVlKSk7XG59XG4iLCAiZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24obm9kZSkge1xuICByZXR1cm4gKG5vZGUub3duZXJEb2N1bWVudCAmJiBub2RlLm93bmVyRG9jdW1lbnQuZGVmYXVsdFZpZXcpIC8vIG5vZGUgaXMgYSBOb2RlXG4gICAgICB8fCAobm9kZS5kb2N1bWVudCAmJiBub2RlKSAvLyBub2RlIGlzIGEgV2luZG93XG4gICAgICB8fCBub2RlLmRlZmF1bHRWaWV3OyAvLyBub2RlIGlzIGEgRG9jdW1lbnRcbn1cbiIsICJpbXBvcnQgZGVmYXVsdFZpZXcgZnJvbSBcIi4uL3dpbmRvdy5qc1wiO1xuXG5mdW5jdGlvbiBzdHlsZVJlbW92ZShuYW1lKSB7XG4gIHJldHVybiBmdW5jdGlvbigpIHtcbiAgICB0aGlzLnN0eWxlLnJlbW92ZVByb3BlcnR5KG5hbWUpO1xuICB9O1xufVxuXG5mdW5jdGlvbiBzdHlsZUNvbnN0YW50KG5hbWUsIHZhbHVlLCBwcmlvcml0eSkge1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgdGhpcy5zdHlsZS5zZXRQcm9wZXJ0eShuYW1lLCB2YWx1ZSwgcHJpb3JpdHkpO1xuICB9O1xufVxuXG5mdW5jdGlvbiBzdHlsZUZ1bmN0aW9uKG5hbWUsIHZhbHVlLCBwcmlvcml0eSkge1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgdmFyIHYgPSB2YWx1ZS5hcHBseSh0aGlzLCBhcmd1bWVudHMpO1xuICAgIGlmICh2ID09IG51bGwpIHRoaXMuc3R5bGUucmVtb3ZlUHJvcGVydHkobmFtZSk7XG4gICAgZWxzZSB0aGlzLnN0eWxlLnNldFByb3BlcnR5KG5hbWUsIHYsIHByaW9yaXR5KTtcbiAgfTtcbn1cblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24obmFtZSwgdmFsdWUsIHByaW9yaXR5KSB7XG4gIHJldHVybiBhcmd1bWVudHMubGVuZ3RoID4gMVxuICAgICAgPyB0aGlzLmVhY2goKHZhbHVlID09IG51bGxcbiAgICAgICAgICAgID8gc3R5bGVSZW1vdmUgOiB0eXBlb2YgdmFsdWUgPT09IFwiZnVuY3Rpb25cIlxuICAgICAgICAgICAgPyBzdHlsZUZ1bmN0aW9uXG4gICAgICAgICAgICA6IHN0eWxlQ29uc3RhbnQpKG5hbWUsIHZhbHVlLCBwcmlvcml0eSA9PSBudWxsID8gXCJcIiA6IHByaW9yaXR5KSlcbiAgICAgIDogc3R5bGVWYWx1ZSh0aGlzLm5vZGUoKSwgbmFtZSk7XG59XG5cbmV4cG9ydCBmdW5jdGlvbiBzdHlsZVZhbHVlKG5vZGUsIG5hbWUpIHtcbiAgcmV0dXJuIG5vZGUuc3R5bGUuZ2V0UHJvcGVydHlWYWx1ZShuYW1lKVxuICAgICAgfHwgZGVmYXVsdFZpZXcobm9kZSkuZ2V0Q29tcHV0ZWRTdHlsZShub2RlLCBudWxsKS5nZXRQcm9wZXJ0eVZhbHVlKG5hbWUpO1xufVxuIiwgImZ1bmN0aW9uIHByb3BlcnR5UmVtb3ZlKG5hbWUpIHtcbiAgcmV0dXJuIGZ1bmN0aW9uKCkge1xuICAgIGRlbGV0ZSB0aGlzW25hbWVdO1xuICB9O1xufVxuXG5mdW5jdGlvbiBwcm9wZXJ0eUNvbnN0YW50KG5hbWUsIHZhbHVlKSB7XG4gIHJldHVybiBmdW5jdGlvbigpIHtcbiAgICB0aGlzW25hbWVdID0gdmFsdWU7XG4gIH07XG59XG5cbmZ1bmN0aW9uIHByb3BlcnR5RnVuY3Rpb24obmFtZSwgdmFsdWUpIHtcbiAgcmV0dXJuIGZ1bmN0aW9uKCkge1xuICAgIHZhciB2ID0gdmFsdWUuYXBwbHkodGhpcywgYXJndW1lbnRzKTtcbiAgICBpZiAodiA9PSBudWxsKSBkZWxldGUgdGhpc1tuYW1lXTtcbiAgICBlbHNlIHRoaXNbbmFtZV0gPSB2O1xuICB9O1xufVxuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbihuYW1lLCB2YWx1ZSkge1xuICByZXR1cm4gYXJndW1lbnRzLmxlbmd0aCA+IDFcbiAgICAgID8gdGhpcy5lYWNoKCh2YWx1ZSA9PSBudWxsXG4gICAgICAgICAgPyBwcm9wZXJ0eVJlbW92ZSA6IHR5cGVvZiB2YWx1ZSA9PT0gXCJmdW5jdGlvblwiXG4gICAgICAgICAgPyBwcm9wZXJ0eUZ1bmN0aW9uXG4gICAgICAgICAgOiBwcm9wZXJ0eUNvbnN0YW50KShuYW1lLCB2YWx1ZSkpXG4gICAgICA6IHRoaXMubm9kZSgpW25hbWVdO1xufVxuIiwgImZ1bmN0aW9uIGNsYXNzQXJyYXkoc3RyaW5nKSB7XG4gIHJldHVybiBzdHJpbmcudHJpbSgpLnNwbGl0KC9efFxccysvKTtcbn1cblxuZnVuY3Rpb24gY2xhc3NMaXN0KG5vZGUpIHtcbiAgcmV0dXJuIG5vZGUuY2xhc3NMaXN0IHx8IG5ldyBDbGFzc0xpc3Qobm9kZSk7XG59XG5cbmZ1bmN0aW9uIENsYXNzTGlzdChub2RlKSB7XG4gIHRoaXMuX25vZGUgPSBub2RlO1xuICB0aGlzLl9uYW1lcyA9IGNsYXNzQXJyYXkobm9kZS5nZXRBdHRyaWJ1dGUoXCJjbGFzc1wiKSB8fCBcIlwiKTtcbn1cblxuQ2xhc3NMaXN0LnByb3RvdHlwZSA9IHtcbiAgYWRkOiBmdW5jdGlvbihuYW1lKSB7XG4gICAgdmFyIGkgPSB0aGlzLl9uYW1lcy5pbmRleE9mKG5hbWUpO1xuICAgIGlmIChpIDwgMCkge1xuICAgICAgdGhpcy5fbmFtZXMucHVzaChuYW1lKTtcbiAgICAgIHRoaXMuX25vZGUuc2V0QXR0cmlidXRlKFwiY2xhc3NcIiwgdGhpcy5fbmFtZXMuam9pbihcIiBcIikpO1xuICAgIH1cbiAgfSxcbiAgcmVtb3ZlOiBmdW5jdGlvbihuYW1lKSB7XG4gICAgdmFyIGkgPSB0aGlzLl9uYW1lcy5pbmRleE9mKG5hbWUpO1xuICAgIGlmIChpID49IDApIHtcbiAgICAgIHRoaXMuX25hbWVzLnNwbGljZShpLCAxKTtcbiAgICAgIHRoaXMuX25vZGUuc2V0QXR0cmlidXRlKFwiY2xhc3NcIiwgdGhpcy5fbmFtZXMuam9pbihcIiBcIikpO1xuICAgIH1cbiAgfSxcbiAgY29udGFpbnM6IGZ1bmN0aW9uKG5hbWUpIHtcbiAgICByZXR1cm4gdGhpcy5fbmFtZXMuaW5kZXhPZihuYW1lKSA+PSAwO1xuICB9XG59O1xuXG5mdW5jdGlvbiBjbGFzc2VkQWRkKG5vZGUsIG5hbWVzKSB7XG4gIHZhciBsaXN0ID0gY2xhc3NMaXN0KG5vZGUpLCBpID0gLTEsIG4gPSBuYW1lcy5sZW5ndGg7XG4gIHdoaWxlICgrK2kgPCBuKSBsaXN0LmFkZChuYW1lc1tpXSk7XG59XG5cbmZ1bmN0aW9uIGNsYXNzZWRSZW1vdmUobm9kZSwgbmFtZXMpIHtcbiAgdmFyIGxpc3QgPSBjbGFzc0xpc3Qobm9kZSksIGkgPSAtMSwgbiA9IG5hbWVzLmxlbmd0aDtcbiAgd2hpbGUgKCsraSA8IG4pIGxpc3QucmVtb3ZlKG5hbWVzW2ldKTtcbn1cblxuZnVuY3Rpb24gY2xhc3NlZFRydWUobmFtZXMpIHtcbiAgcmV0dXJuIGZ1bmN0aW9uKCkge1xuICAgIGNsYXNzZWRBZGQodGhpcywgbmFtZXMpO1xuICB9O1xufVxuXG5mdW5jdGlvbiBjbGFzc2VkRmFsc2UobmFtZXMpIHtcbiAgcmV0dXJuIGZ1bmN0aW9uKCkge1xuICAgIGNsYXNzZWRSZW1vdmUodGhpcywgbmFtZXMpO1xuICB9O1xufVxuXG5mdW5jdGlvbiBjbGFzc2VkRnVuY3Rpb24obmFtZXMsIHZhbHVlKSB7XG4gIHJldHVybiBmdW5jdGlvbigpIHtcbiAgICAodmFsdWUuYXBwbHkodGhpcywgYXJndW1lbnRzKSA/IGNsYXNzZWRBZGQgOiBjbGFzc2VkUmVtb3ZlKSh0aGlzLCBuYW1lcyk7XG4gIH07XG59XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKG5hbWUsIHZhbHVlKSB7XG4gIHZhciBuYW1lcyA9IGNsYXNzQXJyYXkobmFtZSArIFwiXCIpO1xuXG4gIGlmIChhcmd1bWVudHMubGVuZ3RoIDwgMikge1xuICAgIHZhciBsaXN0ID0gY2xhc3NMaXN0KHRoaXMubm9kZSgpKSwgaSA9IC0xLCBuID0gbmFtZXMubGVuZ3RoO1xuICAgIHdoaWxlICgrK2kgPCBuKSBpZiAoIWxpc3QuY29udGFpbnMobmFtZXNbaV0pKSByZXR1cm4gZmFsc2U7XG4gICAgcmV0dXJuIHRydWU7XG4gIH1cblxuICByZXR1cm4gdGhpcy5lYWNoKCh0eXBlb2YgdmFsdWUgPT09IFwiZnVuY3Rpb25cIlxuICAgICAgPyBjbGFzc2VkRnVuY3Rpb24gOiB2YWx1ZVxuICAgICAgPyBjbGFzc2VkVHJ1ZVxuICAgICAgOiBjbGFzc2VkRmFsc2UpKG5hbWVzLCB2YWx1ZSkpO1xufVxuIiwgImZ1bmN0aW9uIHRleHRSZW1vdmUoKSB7XG4gIHRoaXMudGV4dENvbnRlbnQgPSBcIlwiO1xufVxuXG5mdW5jdGlvbiB0ZXh0Q29uc3RhbnQodmFsdWUpIHtcbiAgcmV0dXJuIGZ1bmN0aW9uKCkge1xuICAgIHRoaXMudGV4dENvbnRlbnQgPSB2YWx1ZTtcbiAgfTtcbn1cblxuZnVuY3Rpb24gdGV4dEZ1bmN0aW9uKHZhbHVlKSB7XG4gIHJldHVybiBmdW5jdGlvbigpIHtcbiAgICB2YXIgdiA9IHZhbHVlLmFwcGx5KHRoaXMsIGFyZ3VtZW50cyk7XG4gICAgdGhpcy50ZXh0Q29udGVudCA9IHYgPT0gbnVsbCA/IFwiXCIgOiB2O1xuICB9O1xufVxuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbih2YWx1ZSkge1xuICByZXR1cm4gYXJndW1lbnRzLmxlbmd0aFxuICAgICAgPyB0aGlzLmVhY2godmFsdWUgPT0gbnVsbFxuICAgICAgICAgID8gdGV4dFJlbW92ZSA6ICh0eXBlb2YgdmFsdWUgPT09IFwiZnVuY3Rpb25cIlxuICAgICAgICAgID8gdGV4dEZ1bmN0aW9uXG4gICAgICAgICAgOiB0ZXh0Q29uc3RhbnQpKHZhbHVlKSlcbiAgICAgIDogdGhpcy5ub2RlKCkudGV4dENvbnRlbnQ7XG59XG4iLCAiZnVuY3Rpb24gaHRtbFJlbW92ZSgpIHtcbiAgdGhpcy5pbm5lckhUTUwgPSBcIlwiO1xufVxuXG5mdW5jdGlvbiBodG1sQ29uc3RhbnQodmFsdWUpIHtcbiAgcmV0dXJuIGZ1bmN0aW9uKCkge1xuICAgIHRoaXMuaW5uZXJIVE1MID0gdmFsdWU7XG4gIH07XG59XG5cbmZ1bmN0aW9uIGh0bWxGdW5jdGlvbih2YWx1ZSkge1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgdmFyIHYgPSB2YWx1ZS5hcHBseSh0aGlzLCBhcmd1bWVudHMpO1xuICAgIHRoaXMuaW5uZXJIVE1MID0gdiA9PSBudWxsID8gXCJcIiA6IHY7XG4gIH07XG59XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKHZhbHVlKSB7XG4gIHJldHVybiBhcmd1bWVudHMubGVuZ3RoXG4gICAgICA/IHRoaXMuZWFjaCh2YWx1ZSA9PSBudWxsXG4gICAgICAgICAgPyBodG1sUmVtb3ZlIDogKHR5cGVvZiB2YWx1ZSA9PT0gXCJmdW5jdGlvblwiXG4gICAgICAgICAgPyBodG1sRnVuY3Rpb25cbiAgICAgICAgICA6IGh0bWxDb25zdGFudCkodmFsdWUpKVxuICAgICAgOiB0aGlzLm5vZGUoKS5pbm5lckhUTUw7XG59XG4iLCAiZnVuY3Rpb24gcmFpc2UoKSB7XG4gIGlmICh0aGlzLm5leHRTaWJsaW5nKSB0aGlzLnBhcmVudE5vZGUuYXBwZW5kQ2hpbGQodGhpcyk7XG59XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKCkge1xuICByZXR1cm4gdGhpcy5lYWNoKHJhaXNlKTtcbn1cbiIsICJmdW5jdGlvbiBsb3dlcigpIHtcbiAgaWYgKHRoaXMucHJldmlvdXNTaWJsaW5nKSB0aGlzLnBhcmVudE5vZGUuaW5zZXJ0QmVmb3JlKHRoaXMsIHRoaXMucGFyZW50Tm9kZS5maXJzdENoaWxkKTtcbn1cblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24oKSB7XG4gIHJldHVybiB0aGlzLmVhY2gobG93ZXIpO1xufVxuIiwgImltcG9ydCBjcmVhdG9yIGZyb20gXCIuLi9jcmVhdG9yLmpzXCI7XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKG5hbWUpIHtcbiAgdmFyIGNyZWF0ZSA9IHR5cGVvZiBuYW1lID09PSBcImZ1bmN0aW9uXCIgPyBuYW1lIDogY3JlYXRvcihuYW1lKTtcbiAgcmV0dXJuIHRoaXMuc2VsZWN0KGZ1bmN0aW9uKCkge1xuICAgIHJldHVybiB0aGlzLmFwcGVuZENoaWxkKGNyZWF0ZS5hcHBseSh0aGlzLCBhcmd1bWVudHMpKTtcbiAgfSk7XG59XG4iLCAiaW1wb3J0IGNyZWF0b3IgZnJvbSBcIi4uL2NyZWF0b3IuanNcIjtcbmltcG9ydCBzZWxlY3RvciBmcm9tIFwiLi4vc2VsZWN0b3IuanNcIjtcblxuZnVuY3Rpb24gY29uc3RhbnROdWxsKCkge1xuICByZXR1cm4gbnVsbDtcbn1cblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24obmFtZSwgYmVmb3JlKSB7XG4gIHZhciBjcmVhdGUgPSB0eXBlb2YgbmFtZSA9PT0gXCJmdW5jdGlvblwiID8gbmFtZSA6IGNyZWF0b3IobmFtZSksXG4gICAgICBzZWxlY3QgPSBiZWZvcmUgPT0gbnVsbCA/IGNvbnN0YW50TnVsbCA6IHR5cGVvZiBiZWZvcmUgPT09IFwiZnVuY3Rpb25cIiA/IGJlZm9yZSA6IHNlbGVjdG9yKGJlZm9yZSk7XG4gIHJldHVybiB0aGlzLnNlbGVjdChmdW5jdGlvbigpIHtcbiAgICByZXR1cm4gdGhpcy5pbnNlcnRCZWZvcmUoY3JlYXRlLmFwcGx5KHRoaXMsIGFyZ3VtZW50cyksIHNlbGVjdC5hcHBseSh0aGlzLCBhcmd1bWVudHMpIHx8IG51bGwpO1xuICB9KTtcbn1cbiIsICJmdW5jdGlvbiByZW1vdmUoKSB7XG4gIHZhciBwYXJlbnQgPSB0aGlzLnBhcmVudE5vZGU7XG4gIGlmIChwYXJlbnQpIHBhcmVudC5yZW1vdmVDaGlsZCh0aGlzKTtcbn1cblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24oKSB7XG4gIHJldHVybiB0aGlzLmVhY2gocmVtb3ZlKTtcbn1cbiIsICJmdW5jdGlvbiBzZWxlY3Rpb25fY2xvbmVTaGFsbG93KCkge1xuICB2YXIgY2xvbmUgPSB0aGlzLmNsb25lTm9kZShmYWxzZSksIHBhcmVudCA9IHRoaXMucGFyZW50Tm9kZTtcbiAgcmV0dXJuIHBhcmVudCA/IHBhcmVudC5pbnNlcnRCZWZvcmUoY2xvbmUsIHRoaXMubmV4dFNpYmxpbmcpIDogY2xvbmU7XG59XG5cbmZ1bmN0aW9uIHNlbGVjdGlvbl9jbG9uZURlZXAoKSB7XG4gIHZhciBjbG9uZSA9IHRoaXMuY2xvbmVOb2RlKHRydWUpLCBwYXJlbnQgPSB0aGlzLnBhcmVudE5vZGU7XG4gIHJldHVybiBwYXJlbnQgPyBwYXJlbnQuaW5zZXJ0QmVmb3JlKGNsb25lLCB0aGlzLm5leHRTaWJsaW5nKSA6IGNsb25lO1xufVxuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbihkZWVwKSB7XG4gIHJldHVybiB0aGlzLnNlbGVjdChkZWVwID8gc2VsZWN0aW9uX2Nsb25lRGVlcCA6IHNlbGVjdGlvbl9jbG9uZVNoYWxsb3cpO1xufVxuIiwgImV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKHZhbHVlKSB7XG4gIHJldHVybiBhcmd1bWVudHMubGVuZ3RoXG4gICAgICA/IHRoaXMucHJvcGVydHkoXCJfX2RhdGFfX1wiLCB2YWx1ZSlcbiAgICAgIDogdGhpcy5ub2RlKCkuX19kYXRhX187XG59XG4iLCAiZnVuY3Rpb24gY29udGV4dExpc3RlbmVyKGxpc3RlbmVyKSB7XG4gIHJldHVybiBmdW5jdGlvbihldmVudCkge1xuICAgIGxpc3RlbmVyLmNhbGwodGhpcywgZXZlbnQsIHRoaXMuX19kYXRhX18pO1xuICB9O1xufVxuXG5mdW5jdGlvbiBwYXJzZVR5cGVuYW1lcyh0eXBlbmFtZXMpIHtcbiAgcmV0dXJuIHR5cGVuYW1lcy50cmltKCkuc3BsaXQoL158XFxzKy8pLm1hcChmdW5jdGlvbih0KSB7XG4gICAgdmFyIG5hbWUgPSBcIlwiLCBpID0gdC5pbmRleE9mKFwiLlwiKTtcbiAgICBpZiAoaSA+PSAwKSBuYW1lID0gdC5zbGljZShpICsgMSksIHQgPSB0LnNsaWNlKDAsIGkpO1xuICAgIHJldHVybiB7dHlwZTogdCwgbmFtZTogbmFtZX07XG4gIH0pO1xufVxuXG5mdW5jdGlvbiBvblJlbW92ZSh0eXBlbmFtZSkge1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgdmFyIG9uID0gdGhpcy5fX29uO1xuICAgIGlmICghb24pIHJldHVybjtcbiAgICBmb3IgKHZhciBqID0gMCwgaSA9IC0xLCBtID0gb24ubGVuZ3RoLCBvOyBqIDwgbTsgKytqKSB7XG4gICAgICBpZiAobyA9IG9uW2pdLCAoIXR5cGVuYW1lLnR5cGUgfHwgby50eXBlID09PSB0eXBlbmFtZS50eXBlKSAmJiBvLm5hbWUgPT09IHR5cGVuYW1lLm5hbWUpIHtcbiAgICAgICAgdGhpcy5yZW1vdmVFdmVudExpc3RlbmVyKG8udHlwZSwgby5saXN0ZW5lciwgby5vcHRpb25zKTtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIG9uWysraV0gPSBvO1xuICAgICAgfVxuICAgIH1cbiAgICBpZiAoKytpKSBvbi5sZW5ndGggPSBpO1xuICAgIGVsc2UgZGVsZXRlIHRoaXMuX19vbjtcbiAgfTtcbn1cblxuZnVuY3Rpb24gb25BZGQodHlwZW5hbWUsIHZhbHVlLCBvcHRpb25zKSB7XG4gIHJldHVybiBmdW5jdGlvbigpIHtcbiAgICB2YXIgb24gPSB0aGlzLl9fb24sIG8sIGxpc3RlbmVyID0gY29udGV4dExpc3RlbmVyKHZhbHVlKTtcbiAgICBpZiAob24pIGZvciAodmFyIGogPSAwLCBtID0gb24ubGVuZ3RoOyBqIDwgbTsgKytqKSB7XG4gICAgICBpZiAoKG8gPSBvbltqXSkudHlwZSA9PT0gdHlwZW5hbWUudHlwZSAmJiBvLm5hbWUgPT09IHR5cGVuYW1lLm5hbWUpIHtcbiAgICAgICAgdGhpcy5yZW1vdmVFdmVudExpc3RlbmVyKG8udHlwZSwgby5saXN0ZW5lciwgby5vcHRpb25zKTtcbiAgICAgICAgdGhpcy5hZGRFdmVudExpc3RlbmVyKG8udHlwZSwgby5saXN0ZW5lciA9IGxpc3RlbmVyLCBvLm9wdGlvbnMgPSBvcHRpb25zKTtcbiAgICAgICAgby52YWx1ZSA9IHZhbHVlO1xuICAgICAgICByZXR1cm47XG4gICAgICB9XG4gICAgfVxuICAgIHRoaXMuYWRkRXZlbnRMaXN0ZW5lcih0eXBlbmFtZS50eXBlLCBsaXN0ZW5lciwgb3B0aW9ucyk7XG4gICAgbyA9IHt0eXBlOiB0eXBlbmFtZS50eXBlLCBuYW1lOiB0eXBlbmFtZS5uYW1lLCB2YWx1ZTogdmFsdWUsIGxpc3RlbmVyOiBsaXN0ZW5lciwgb3B0aW9uczogb3B0aW9uc307XG4gICAgaWYgKCFvbikgdGhpcy5fX29uID0gW29dO1xuICAgIGVsc2Ugb24ucHVzaChvKTtcbiAgfTtcbn1cblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24odHlwZW5hbWUsIHZhbHVlLCBvcHRpb25zKSB7XG4gIHZhciB0eXBlbmFtZXMgPSBwYXJzZVR5cGVuYW1lcyh0eXBlbmFtZSArIFwiXCIpLCBpLCBuID0gdHlwZW5hbWVzLmxlbmd0aCwgdDtcblxuICBpZiAoYXJndW1lbnRzLmxlbmd0aCA8IDIpIHtcbiAgICB2YXIgb24gPSB0aGlzLm5vZGUoKS5fX29uO1xuICAgIGlmIChvbikgZm9yICh2YXIgaiA9IDAsIG0gPSBvbi5sZW5ndGgsIG87IGogPCBtOyArK2opIHtcbiAgICAgIGZvciAoaSA9IDAsIG8gPSBvbltqXTsgaSA8IG47ICsraSkge1xuICAgICAgICBpZiAoKHQgPSB0eXBlbmFtZXNbaV0pLnR5cGUgPT09IG8udHlwZSAmJiB0Lm5hbWUgPT09IG8ubmFtZSkge1xuICAgICAgICAgIHJldHVybiBvLnZhbHVlO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybjtcbiAgfVxuXG4gIG9uID0gdmFsdWUgPyBvbkFkZCA6IG9uUmVtb3ZlO1xuICBmb3IgKGkgPSAwOyBpIDwgbjsgKytpKSB0aGlzLmVhY2gob24odHlwZW5hbWVzW2ldLCB2YWx1ZSwgb3B0aW9ucykpO1xuICByZXR1cm4gdGhpcztcbn1cbiIsICJpbXBvcnQgZGVmYXVsdFZpZXcgZnJvbSBcIi4uL3dpbmRvdy5qc1wiO1xuXG5mdW5jdGlvbiBkaXNwYXRjaEV2ZW50KG5vZGUsIHR5cGUsIHBhcmFtcykge1xuICB2YXIgd2luZG93ID0gZGVmYXVsdFZpZXcobm9kZSksXG4gICAgICBldmVudCA9IHdpbmRvdy5DdXN0b21FdmVudDtcblxuICBpZiAodHlwZW9mIGV2ZW50ID09PSBcImZ1bmN0aW9uXCIpIHtcbiAgICBldmVudCA9IG5ldyBldmVudCh0eXBlLCBwYXJhbXMpO1xuICB9IGVsc2Uge1xuICAgIGV2ZW50ID0gd2luZG93LmRvY3VtZW50LmNyZWF0ZUV2ZW50KFwiRXZlbnRcIik7XG4gICAgaWYgKHBhcmFtcykgZXZlbnQuaW5pdEV2ZW50KHR5cGUsIHBhcmFtcy5idWJibGVzLCBwYXJhbXMuY2FuY2VsYWJsZSksIGV2ZW50LmRldGFpbCA9IHBhcmFtcy5kZXRhaWw7XG4gICAgZWxzZSBldmVudC5pbml0RXZlbnQodHlwZSwgZmFsc2UsIGZhbHNlKTtcbiAgfVxuXG4gIG5vZGUuZGlzcGF0Y2hFdmVudChldmVudCk7XG59XG5cbmZ1bmN0aW9uIGRpc3BhdGNoQ29uc3RhbnQodHlwZSwgcGFyYW1zKSB7XG4gIHJldHVybiBmdW5jdGlvbigpIHtcbiAgICByZXR1cm4gZGlzcGF0Y2hFdmVudCh0aGlzLCB0eXBlLCBwYXJhbXMpO1xuICB9O1xufVxuXG5mdW5jdGlvbiBkaXNwYXRjaEZ1bmN0aW9uKHR5cGUsIHBhcmFtcykge1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgcmV0dXJuIGRpc3BhdGNoRXZlbnQodGhpcywgdHlwZSwgcGFyYW1zLmFwcGx5KHRoaXMsIGFyZ3VtZW50cykpO1xuICB9O1xufVxuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbih0eXBlLCBwYXJhbXMpIHtcbiAgcmV0dXJuIHRoaXMuZWFjaCgodHlwZW9mIHBhcmFtcyA9PT0gXCJmdW5jdGlvblwiXG4gICAgICA/IGRpc3BhdGNoRnVuY3Rpb25cbiAgICAgIDogZGlzcGF0Y2hDb25zdGFudCkodHlwZSwgcGFyYW1zKSk7XG59XG4iLCAiZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24qKCkge1xuICBmb3IgKHZhciBncm91cHMgPSB0aGlzLl9ncm91cHMsIGogPSAwLCBtID0gZ3JvdXBzLmxlbmd0aDsgaiA8IG07ICsraikge1xuICAgIGZvciAodmFyIGdyb3VwID0gZ3JvdXBzW2pdLCBpID0gMCwgbiA9IGdyb3VwLmxlbmd0aCwgbm9kZTsgaSA8IG47ICsraSkge1xuICAgICAgaWYgKG5vZGUgPSBncm91cFtpXSkgeWllbGQgbm9kZTtcbiAgICB9XG4gIH1cbn1cbiIsICJpbXBvcnQgc2VsZWN0aW9uX3NlbGVjdCBmcm9tIFwiLi9zZWxlY3QuanNcIjtcbmltcG9ydCBzZWxlY3Rpb25fc2VsZWN0QWxsIGZyb20gXCIuL3NlbGVjdEFsbC5qc1wiO1xuaW1wb3J0IHNlbGVjdGlvbl9zZWxlY3RDaGlsZCBmcm9tIFwiLi9zZWxlY3RDaGlsZC5qc1wiO1xuaW1wb3J0IHNlbGVjdGlvbl9zZWxlY3RDaGlsZHJlbiBmcm9tIFwiLi9zZWxlY3RDaGlsZHJlbi5qc1wiO1xuaW1wb3J0IHNlbGVjdGlvbl9maWx0ZXIgZnJvbSBcIi4vZmlsdGVyLmpzXCI7XG5pbXBvcnQgc2VsZWN0aW9uX2RhdGEgZnJvbSBcIi4vZGF0YS5qc1wiO1xuaW1wb3J0IHNlbGVjdGlvbl9lbnRlciBmcm9tIFwiLi9lbnRlci5qc1wiO1xuaW1wb3J0IHNlbGVjdGlvbl9leGl0IGZyb20gXCIuL2V4aXQuanNcIjtcbmltcG9ydCBzZWxlY3Rpb25fam9pbiBmcm9tIFwiLi9qb2luLmpzXCI7XG5pbXBvcnQgc2VsZWN0aW9uX21lcmdlIGZyb20gXCIuL21lcmdlLmpzXCI7XG5pbXBvcnQgc2VsZWN0aW9uX29yZGVyIGZyb20gXCIuL29yZGVyLmpzXCI7XG5pbXBvcnQgc2VsZWN0aW9uX3NvcnQgZnJvbSBcIi4vc29ydC5qc1wiO1xuaW1wb3J0IHNlbGVjdGlvbl9jYWxsIGZyb20gXCIuL2NhbGwuanNcIjtcbmltcG9ydCBzZWxlY3Rpb25fbm9kZXMgZnJvbSBcIi4vbm9kZXMuanNcIjtcbmltcG9ydCBzZWxlY3Rpb25fbm9kZSBmcm9tIFwiLi9ub2RlLmpzXCI7XG5pbXBvcnQgc2VsZWN0aW9uX3NpemUgZnJvbSBcIi4vc2l6ZS5qc1wiO1xuaW1wb3J0IHNlbGVjdGlvbl9lbXB0eSBmcm9tIFwiLi9lbXB0eS5qc1wiO1xuaW1wb3J0IHNlbGVjdGlvbl9lYWNoIGZyb20gXCIuL2VhY2guanNcIjtcbmltcG9ydCBzZWxlY3Rpb25fYXR0ciBmcm9tIFwiLi9hdHRyLmpzXCI7XG5pbXBvcnQgc2VsZWN0aW9uX3N0eWxlIGZyb20gXCIuL3N0eWxlLmpzXCI7XG5pbXBvcnQgc2VsZWN0aW9uX3Byb3BlcnR5IGZyb20gXCIuL3Byb3BlcnR5LmpzXCI7XG5pbXBvcnQgc2VsZWN0aW9uX2NsYXNzZWQgZnJvbSBcIi4vY2xhc3NlZC5qc1wiO1xuaW1wb3J0IHNlbGVjdGlvbl90ZXh0IGZyb20gXCIuL3RleHQuanNcIjtcbmltcG9ydCBzZWxlY3Rpb25faHRtbCBmcm9tIFwiLi9odG1sLmpzXCI7XG5pbXBvcnQgc2VsZWN0aW9uX3JhaXNlIGZyb20gXCIuL3JhaXNlLmpzXCI7XG5pbXBvcnQgc2VsZWN0aW9uX2xvd2VyIGZyb20gXCIuL2xvd2VyLmpzXCI7XG5pbXBvcnQgc2VsZWN0aW9uX2FwcGVuZCBmcm9tIFwiLi9hcHBlbmQuanNcIjtcbmltcG9ydCBzZWxlY3Rpb25faW5zZXJ0IGZyb20gXCIuL2luc2VydC5qc1wiO1xuaW1wb3J0IHNlbGVjdGlvbl9yZW1vdmUgZnJvbSBcIi4vcmVtb3ZlLmpzXCI7XG5pbXBvcnQgc2VsZWN0aW9uX2Nsb25lIGZyb20gXCIuL2Nsb25lLmpzXCI7XG5pbXBvcnQgc2VsZWN0aW9uX2RhdHVtIGZyb20gXCIuL2RhdHVtLmpzXCI7XG5pbXBvcnQgc2VsZWN0aW9uX29uIGZyb20gXCIuL29uLmpzXCI7XG5pbXBvcnQgc2VsZWN0aW9uX2Rpc3BhdGNoIGZyb20gXCIuL2Rpc3BhdGNoLmpzXCI7XG5pbXBvcnQgc2VsZWN0aW9uX2l0ZXJhdG9yIGZyb20gXCIuL2l0ZXJhdG9yLmpzXCI7XG5cbmV4cG9ydCB2YXIgcm9vdCA9IFtudWxsXTtcblxuZXhwb3J0IGZ1bmN0aW9uIFNlbGVjdGlvbihncm91cHMsIHBhcmVudHMpIHtcbiAgdGhpcy5fZ3JvdXBzID0gZ3JvdXBzO1xuICB0aGlzLl9wYXJlbnRzID0gcGFyZW50cztcbn1cblxuZnVuY3Rpb24gc2VsZWN0aW9uKCkge1xuICByZXR1cm4gbmV3IFNlbGVjdGlvbihbW2RvY3VtZW50LmRvY3VtZW50RWxlbWVudF1dLCByb290KTtcbn1cblxuZnVuY3Rpb24gc2VsZWN0aW9uX3NlbGVjdGlvbigpIHtcbiAgcmV0dXJuIHRoaXM7XG59XG5cblNlbGVjdGlvbi5wcm90b3R5cGUgPSBzZWxlY3Rpb24ucHJvdG90eXBlID0ge1xuICBjb25zdHJ1Y3RvcjogU2VsZWN0aW9uLFxuICBzZWxlY3Q6IHNlbGVjdGlvbl9zZWxlY3QsXG4gIHNlbGVjdEFsbDogc2VsZWN0aW9uX3NlbGVjdEFsbCxcbiAgc2VsZWN0Q2hpbGQ6IHNlbGVjdGlvbl9zZWxlY3RDaGlsZCxcbiAgc2VsZWN0Q2hpbGRyZW46IHNlbGVjdGlvbl9zZWxlY3RDaGlsZHJlbixcbiAgZmlsdGVyOiBzZWxlY3Rpb25fZmlsdGVyLFxuICBkYXRhOiBzZWxlY3Rpb25fZGF0YSxcbiAgZW50ZXI6IHNlbGVjdGlvbl9lbnRlcixcbiAgZXhpdDogc2VsZWN0aW9uX2V4aXQsXG4gIGpvaW46IHNlbGVjdGlvbl9qb2luLFxuICBtZXJnZTogc2VsZWN0aW9uX21lcmdlLFxuICBzZWxlY3Rpb246IHNlbGVjdGlvbl9zZWxlY3Rpb24sXG4gIG9yZGVyOiBzZWxlY3Rpb25fb3JkZXIsXG4gIHNvcnQ6IHNlbGVjdGlvbl9zb3J0LFxuICBjYWxsOiBzZWxlY3Rpb25fY2FsbCxcbiAgbm9kZXM6IHNlbGVjdGlvbl9ub2RlcyxcbiAgbm9kZTogc2VsZWN0aW9uX25vZGUsXG4gIHNpemU6IHNlbGVjdGlvbl9zaXplLFxuICBlbXB0eTogc2VsZWN0aW9uX2VtcHR5LFxuICBlYWNoOiBzZWxlY3Rpb25fZWFjaCxcbiAgYXR0cjogc2VsZWN0aW9uX2F0dHIsXG4gIHN0eWxlOiBzZWxlY3Rpb25fc3R5bGUsXG4gIHByb3BlcnR5OiBzZWxlY3Rpb25fcHJvcGVydHksXG4gIGNsYXNzZWQ6IHNlbGVjdGlvbl9jbGFzc2VkLFxuICB0ZXh0OiBzZWxlY3Rpb25fdGV4dCxcbiAgaHRtbDogc2VsZWN0aW9uX2h0bWwsXG4gIHJhaXNlOiBzZWxlY3Rpb25fcmFpc2UsXG4gIGxvd2VyOiBzZWxlY3Rpb25fbG93ZXIsXG4gIGFwcGVuZDogc2VsZWN0aW9uX2FwcGVuZCxcbiAgaW5zZXJ0OiBzZWxlY3Rpb25faW5zZXJ0LFxuICByZW1vdmU6IHNlbGVjdGlvbl9yZW1vdmUsXG4gIGNsb25lOiBzZWxlY3Rpb25fY2xvbmUsXG4gIGRhdHVtOiBzZWxlY3Rpb25fZGF0dW0sXG4gIG9uOiBzZWxlY3Rpb25fb24sXG4gIGRpc3BhdGNoOiBzZWxlY3Rpb25fZGlzcGF0Y2gsXG4gIFtTeW1ib2wuaXRlcmF0b3JdOiBzZWxlY3Rpb25faXRlcmF0b3Jcbn07XG5cbmV4cG9ydCBkZWZhdWx0IHNlbGVjdGlvbjtcbiIsICJleHBvcnQgZGVmYXVsdCBmdW5jdGlvbihjb25zdHJ1Y3RvciwgZmFjdG9yeSwgcHJvdG90eXBlKSB7XG4gIGNvbnN0cnVjdG9yLnByb3RvdHlwZSA9IGZhY3RvcnkucHJvdG90eXBlID0gcHJvdG90eXBlO1xuICBwcm90b3R5cGUuY29uc3RydWN0b3IgPSBjb25zdHJ1Y3Rvcjtcbn1cblxuZXhwb3J0IGZ1bmN0aW9uIGV4dGVuZChwYXJlbnQsIGRlZmluaXRpb24pIHtcbiAgdmFyIHByb3RvdHlwZSA9IE9iamVjdC5jcmVhdGUocGFyZW50LnByb3RvdHlwZSk7XG4gIGZvciAodmFyIGtleSBpbiBkZWZpbml0aW9uKSBwcm90b3R5cGVba2V5XSA9IGRlZmluaXRpb25ba2V5XTtcbiAgcmV0dXJuIHByb3RvdHlwZTtcbn1cbiIsICJpbXBvcnQgZGVmaW5lLCB7ZXh0ZW5kfSBmcm9tIFwiLi9kZWZpbmUuanNcIjtcblxuZXhwb3J0IGZ1bmN0aW9uIENvbG9yKCkge31cblxuZXhwb3J0IHZhciBkYXJrZXIgPSAwLjc7XG5leHBvcnQgdmFyIGJyaWdodGVyID0gMSAvIGRhcmtlcjtcblxudmFyIHJlSSA9IFwiXFxcXHMqKFsrLV0/XFxcXGQrKVxcXFxzKlwiLFxuICAgIHJlTiA9IFwiXFxcXHMqKFsrLV0/KD86XFxcXGQqXFxcXC4pP1xcXFxkKyg/OltlRV1bKy1dP1xcXFxkKyk/KVxcXFxzKlwiLFxuICAgIHJlUCA9IFwiXFxcXHMqKFsrLV0/KD86XFxcXGQqXFxcXC4pP1xcXFxkKyg/OltlRV1bKy1dP1xcXFxkKyk/KSVcXFxccypcIixcbiAgICByZUhleCA9IC9eIyhbMC05YS1mXXszLDh9KSQvLFxuICAgIHJlUmdiSW50ZWdlciA9IG5ldyBSZWdFeHAoYF5yZ2JcXFxcKCR7cmVJfSwke3JlSX0sJHtyZUl9XFxcXCkkYCksXG4gICAgcmVSZ2JQZXJjZW50ID0gbmV3IFJlZ0V4cChgXnJnYlxcXFwoJHtyZVB9LCR7cmVQfSwke3JlUH1cXFxcKSRgKSxcbiAgICByZVJnYmFJbnRlZ2VyID0gbmV3IFJlZ0V4cChgXnJnYmFcXFxcKCR7cmVJfSwke3JlSX0sJHtyZUl9LCR7cmVOfVxcXFwpJGApLFxuICAgIHJlUmdiYVBlcmNlbnQgPSBuZXcgUmVnRXhwKGBecmdiYVxcXFwoJHtyZVB9LCR7cmVQfSwke3JlUH0sJHtyZU59XFxcXCkkYCksXG4gICAgcmVIc2xQZXJjZW50ID0gbmV3IFJlZ0V4cChgXmhzbFxcXFwoJHtyZU59LCR7cmVQfSwke3JlUH1cXFxcKSRgKSxcbiAgICByZUhzbGFQZXJjZW50ID0gbmV3IFJlZ0V4cChgXmhzbGFcXFxcKCR7cmVOfSwke3JlUH0sJHtyZVB9LCR7cmVOfVxcXFwpJGApO1xuXG52YXIgbmFtZWQgPSB7XG4gIGFsaWNlYmx1ZTogMHhmMGY4ZmYsXG4gIGFudGlxdWV3aGl0ZTogMHhmYWViZDcsXG4gIGFxdWE6IDB4MDBmZmZmLFxuICBhcXVhbWFyaW5lOiAweDdmZmZkNCxcbiAgYXp1cmU6IDB4ZjBmZmZmLFxuICBiZWlnZTogMHhmNWY1ZGMsXG4gIGJpc3F1ZTogMHhmZmU0YzQsXG4gIGJsYWNrOiAweDAwMDAwMCxcbiAgYmxhbmNoZWRhbG1vbmQ6IDB4ZmZlYmNkLFxuICBibHVlOiAweDAwMDBmZixcbiAgYmx1ZXZpb2xldDogMHg4YTJiZTIsXG4gIGJyb3duOiAweGE1MmEyYSxcbiAgYnVybHl3b29kOiAweGRlYjg4NyxcbiAgY2FkZXRibHVlOiAweDVmOWVhMCxcbiAgY2hhcnRyZXVzZTogMHg3ZmZmMDAsXG4gIGNob2NvbGF0ZTogMHhkMjY5MWUsXG4gIGNvcmFsOiAweGZmN2Y1MCxcbiAgY29ybmZsb3dlcmJsdWU6IDB4NjQ5NWVkLFxuICBjb3Juc2lsazogMHhmZmY4ZGMsXG4gIGNyaW1zb246IDB4ZGMxNDNjLFxuICBjeWFuOiAweDAwZmZmZixcbiAgZGFya2JsdWU6IDB4MDAwMDhiLFxuICBkYXJrY3lhbjogMHgwMDhiOGIsXG4gIGRhcmtnb2xkZW5yb2Q6IDB4Yjg4NjBiLFxuICBkYXJrZ3JheTogMHhhOWE5YTksXG4gIGRhcmtncmVlbjogMHgwMDY0MDAsXG4gIGRhcmtncmV5OiAweGE5YTlhOSxcbiAgZGFya2toYWtpOiAweGJkYjc2YixcbiAgZGFya21hZ2VudGE6IDB4OGIwMDhiLFxuICBkYXJrb2xpdmVncmVlbjogMHg1NTZiMmYsXG4gIGRhcmtvcmFuZ2U6IDB4ZmY4YzAwLFxuICBkYXJrb3JjaGlkOiAweDk5MzJjYyxcbiAgZGFya3JlZDogMHg4YjAwMDAsXG4gIGRhcmtzYWxtb246IDB4ZTk5NjdhLFxuICBkYXJrc2VhZ3JlZW46IDB4OGZiYzhmLFxuICBkYXJrc2xhdGVibHVlOiAweDQ4M2Q4YixcbiAgZGFya3NsYXRlZ3JheTogMHgyZjRmNGYsXG4gIGRhcmtzbGF0ZWdyZXk6IDB4MmY0ZjRmLFxuICBkYXJrdHVycXVvaXNlOiAweDAwY2VkMSxcbiAgZGFya3Zpb2xldDogMHg5NDAwZDMsXG4gIGRlZXBwaW5rOiAweGZmMTQ5MyxcbiAgZGVlcHNreWJsdWU6IDB4MDBiZmZmLFxuICBkaW1ncmF5OiAweDY5Njk2OSxcbiAgZGltZ3JleTogMHg2OTY5NjksXG4gIGRvZGdlcmJsdWU6IDB4MWU5MGZmLFxuICBmaXJlYnJpY2s6IDB4YjIyMjIyLFxuICBmbG9yYWx3aGl0ZTogMHhmZmZhZjAsXG4gIGZvcmVzdGdyZWVuOiAweDIyOGIyMixcbiAgZnVjaHNpYTogMHhmZjAwZmYsXG4gIGdhaW5zYm9ybzogMHhkY2RjZGMsXG4gIGdob3N0d2hpdGU6IDB4ZjhmOGZmLFxuICBnb2xkOiAweGZmZDcwMCxcbiAgZ29sZGVucm9kOiAweGRhYTUyMCxcbiAgZ3JheTogMHg4MDgwODAsXG4gIGdyZWVuOiAweDAwODAwMCxcbiAgZ3JlZW55ZWxsb3c6IDB4YWRmZjJmLFxuICBncmV5OiAweDgwODA4MCxcbiAgaG9uZXlkZXc6IDB4ZjBmZmYwLFxuICBob3RwaW5rOiAweGZmNjliNCxcbiAgaW5kaWFucmVkOiAweGNkNWM1YyxcbiAgaW5kaWdvOiAweDRiMDA4MixcbiAgaXZvcnk6IDB4ZmZmZmYwLFxuICBraGFraTogMHhmMGU2OGMsXG4gIGxhdmVuZGVyOiAweGU2ZTZmYSxcbiAgbGF2ZW5kZXJibHVzaDogMHhmZmYwZjUsXG4gIGxhd25ncmVlbjogMHg3Y2ZjMDAsXG4gIGxlbW9uY2hpZmZvbjogMHhmZmZhY2QsXG4gIGxpZ2h0Ymx1ZTogMHhhZGQ4ZTYsXG4gIGxpZ2h0Y29yYWw6IDB4ZjA4MDgwLFxuICBsaWdodGN5YW46IDB4ZTBmZmZmLFxuICBsaWdodGdvbGRlbnJvZHllbGxvdzogMHhmYWZhZDIsXG4gIGxpZ2h0Z3JheTogMHhkM2QzZDMsXG4gIGxpZ2h0Z3JlZW46IDB4OTBlZTkwLFxuICBsaWdodGdyZXk6IDB4ZDNkM2QzLFxuICBsaWdodHBpbms6IDB4ZmZiNmMxLFxuICBsaWdodHNhbG1vbjogMHhmZmEwN2EsXG4gIGxpZ2h0c2VhZ3JlZW46IDB4MjBiMmFhLFxuICBsaWdodHNreWJsdWU6IDB4ODdjZWZhLFxuICBsaWdodHNsYXRlZ3JheTogMHg3Nzg4OTksXG4gIGxpZ2h0c2xhdGVncmV5OiAweDc3ODg5OSxcbiAgbGlnaHRzdGVlbGJsdWU6IDB4YjBjNGRlLFxuICBsaWdodHllbGxvdzogMHhmZmZmZTAsXG4gIGxpbWU6IDB4MDBmZjAwLFxuICBsaW1lZ3JlZW46IDB4MzJjZDMyLFxuICBsaW5lbjogMHhmYWYwZTYsXG4gIG1hZ2VudGE6IDB4ZmYwMGZmLFxuICBtYXJvb246IDB4ODAwMDAwLFxuICBtZWRpdW1hcXVhbWFyaW5lOiAweDY2Y2RhYSxcbiAgbWVkaXVtYmx1ZTogMHgwMDAwY2QsXG4gIG1lZGl1bW9yY2hpZDogMHhiYTU1ZDMsXG4gIG1lZGl1bXB1cnBsZTogMHg5MzcwZGIsXG4gIG1lZGl1bXNlYWdyZWVuOiAweDNjYjM3MSxcbiAgbWVkaXVtc2xhdGVibHVlOiAweDdiNjhlZSxcbiAgbWVkaXVtc3ByaW5nZ3JlZW46IDB4MDBmYTlhLFxuICBtZWRpdW10dXJxdW9pc2U6IDB4NDhkMWNjLFxuICBtZWRpdW12aW9sZXRyZWQ6IDB4YzcxNTg1LFxuICBtaWRuaWdodGJsdWU6IDB4MTkxOTcwLFxuICBtaW50Y3JlYW06IDB4ZjVmZmZhLFxuICBtaXN0eXJvc2U6IDB4ZmZlNGUxLFxuICBtb2NjYXNpbjogMHhmZmU0YjUsXG4gIG5hdmFqb3doaXRlOiAweGZmZGVhZCxcbiAgbmF2eTogMHgwMDAwODAsXG4gIG9sZGxhY2U6IDB4ZmRmNWU2LFxuICBvbGl2ZTogMHg4MDgwMDAsXG4gIG9saXZlZHJhYjogMHg2YjhlMjMsXG4gIG9yYW5nZTogMHhmZmE1MDAsXG4gIG9yYW5nZXJlZDogMHhmZjQ1MDAsXG4gIG9yY2hpZDogMHhkYTcwZDYsXG4gIHBhbGVnb2xkZW5yb2Q6IDB4ZWVlOGFhLFxuICBwYWxlZ3JlZW46IDB4OThmYjk4LFxuICBwYWxldHVycXVvaXNlOiAweGFmZWVlZSxcbiAgcGFsZXZpb2xldHJlZDogMHhkYjcwOTMsXG4gIHBhcGF5YXdoaXA6IDB4ZmZlZmQ1LFxuICBwZWFjaHB1ZmY6IDB4ZmZkYWI5LFxuICBwZXJ1OiAweGNkODUzZixcbiAgcGluazogMHhmZmMwY2IsXG4gIHBsdW06IDB4ZGRhMGRkLFxuICBwb3dkZXJibHVlOiAweGIwZTBlNixcbiAgcHVycGxlOiAweDgwMDA4MCxcbiAgcmViZWNjYXB1cnBsZTogMHg2NjMzOTksXG4gIHJlZDogMHhmZjAwMDAsXG4gIHJvc3licm93bjogMHhiYzhmOGYsXG4gIHJveWFsYmx1ZTogMHg0MTY5ZTEsXG4gIHNhZGRsZWJyb3duOiAweDhiNDUxMyxcbiAgc2FsbW9uOiAweGZhODA3MixcbiAgc2FuZHlicm93bjogMHhmNGE0NjAsXG4gIHNlYWdyZWVuOiAweDJlOGI1NyxcbiAgc2Vhc2hlbGw6IDB4ZmZmNWVlLFxuICBzaWVubmE6IDB4YTA1MjJkLFxuICBzaWx2ZXI6IDB4YzBjMGMwLFxuICBza3libHVlOiAweDg3Y2VlYixcbiAgc2xhdGVibHVlOiAweDZhNWFjZCxcbiAgc2xhdGVncmF5OiAweDcwODA5MCxcbiAgc2xhdGVncmV5OiAweDcwODA5MCxcbiAgc25vdzogMHhmZmZhZmEsXG4gIHNwcmluZ2dyZWVuOiAweDAwZmY3ZixcbiAgc3RlZWxibHVlOiAweDQ2ODJiNCxcbiAgdGFuOiAweGQyYjQ4YyxcbiAgdGVhbDogMHgwMDgwODAsXG4gIHRoaXN0bGU6IDB4ZDhiZmQ4LFxuICB0b21hdG86IDB4ZmY2MzQ3LFxuICB0dXJxdW9pc2U6IDB4NDBlMGQwLFxuICB2aW9sZXQ6IDB4ZWU4MmVlLFxuICB3aGVhdDogMHhmNWRlYjMsXG4gIHdoaXRlOiAweGZmZmZmZixcbiAgd2hpdGVzbW9rZTogMHhmNWY1ZjUsXG4gIHllbGxvdzogMHhmZmZmMDAsXG4gIHllbGxvd2dyZWVuOiAweDlhY2QzMlxufTtcblxuZGVmaW5lKENvbG9yLCBjb2xvciwge1xuICBjb3B5KGNoYW5uZWxzKSB7XG4gICAgcmV0dXJuIE9iamVjdC5hc3NpZ24obmV3IHRoaXMuY29uc3RydWN0b3IsIHRoaXMsIGNoYW5uZWxzKTtcbiAgfSxcbiAgZGlzcGxheWFibGUoKSB7XG4gICAgcmV0dXJuIHRoaXMucmdiKCkuZGlzcGxheWFibGUoKTtcbiAgfSxcbiAgaGV4OiBjb2xvcl9mb3JtYXRIZXgsIC8vIERlcHJlY2F0ZWQhIFVzZSBjb2xvci5mb3JtYXRIZXguXG4gIGZvcm1hdEhleDogY29sb3JfZm9ybWF0SGV4LFxuICBmb3JtYXRIZXg4OiBjb2xvcl9mb3JtYXRIZXg4LFxuICBmb3JtYXRIc2w6IGNvbG9yX2Zvcm1hdEhzbCxcbiAgZm9ybWF0UmdiOiBjb2xvcl9mb3JtYXRSZ2IsXG4gIHRvU3RyaW5nOiBjb2xvcl9mb3JtYXRSZ2Jcbn0pO1xuXG5mdW5jdGlvbiBjb2xvcl9mb3JtYXRIZXgoKSB7XG4gIHJldHVybiB0aGlzLnJnYigpLmZvcm1hdEhleCgpO1xufVxuXG5mdW5jdGlvbiBjb2xvcl9mb3JtYXRIZXg4KCkge1xuICByZXR1cm4gdGhpcy5yZ2IoKS5mb3JtYXRIZXg4KCk7XG59XG5cbmZ1bmN0aW9uIGNvbG9yX2Zvcm1hdEhzbCgpIHtcbiAgcmV0dXJuIGhzbENvbnZlcnQodGhpcykuZm9ybWF0SHNsKCk7XG59XG5cbmZ1bmN0aW9uIGNvbG9yX2Zvcm1hdFJnYigpIHtcbiAgcmV0dXJuIHRoaXMucmdiKCkuZm9ybWF0UmdiKCk7XG59XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uIGNvbG9yKGZvcm1hdCkge1xuICB2YXIgbSwgbDtcbiAgZm9ybWF0ID0gKGZvcm1hdCArIFwiXCIpLnRyaW0oKS50b0xvd2VyQ2FzZSgpO1xuICByZXR1cm4gKG0gPSByZUhleC5leGVjKGZvcm1hdCkpID8gKGwgPSBtWzFdLmxlbmd0aCwgbSA9IHBhcnNlSW50KG1bMV0sIDE2KSwgbCA9PT0gNiA/IHJnYm4obSkgLy8gI2ZmMDAwMFxuICAgICAgOiBsID09PSAzID8gbmV3IFJnYigobSA+PiA4ICYgMHhmKSB8IChtID4+IDQgJiAweGYwKSwgKG0gPj4gNCAmIDB4ZikgfCAobSAmIDB4ZjApLCAoKG0gJiAweGYpIDw8IDQpIHwgKG0gJiAweGYpLCAxKSAvLyAjZjAwXG4gICAgICA6IGwgPT09IDggPyByZ2JhKG0gPj4gMjQgJiAweGZmLCBtID4+IDE2ICYgMHhmZiwgbSA+PiA4ICYgMHhmZiwgKG0gJiAweGZmKSAvIDB4ZmYpIC8vICNmZjAwMDAwMFxuICAgICAgOiBsID09PSA0ID8gcmdiYSgobSA+PiAxMiAmIDB4ZikgfCAobSA+PiA4ICYgMHhmMCksIChtID4+IDggJiAweGYpIHwgKG0gPj4gNCAmIDB4ZjApLCAobSA+PiA0ICYgMHhmKSB8IChtICYgMHhmMCksICgoKG0gJiAweGYpIDw8IDQpIHwgKG0gJiAweGYpKSAvIDB4ZmYpIC8vICNmMDAwXG4gICAgICA6IG51bGwpIC8vIGludmFsaWQgaGV4XG4gICAgICA6IChtID0gcmVSZ2JJbnRlZ2VyLmV4ZWMoZm9ybWF0KSkgPyBuZXcgUmdiKG1bMV0sIG1bMl0sIG1bM10sIDEpIC8vIHJnYigyNTUsIDAsIDApXG4gICAgICA6IChtID0gcmVSZ2JQZXJjZW50LmV4ZWMoZm9ybWF0KSkgPyBuZXcgUmdiKG1bMV0gKiAyNTUgLyAxMDAsIG1bMl0gKiAyNTUgLyAxMDAsIG1bM10gKiAyNTUgLyAxMDAsIDEpIC8vIHJnYigxMDAlLCAwJSwgMCUpXG4gICAgICA6IChtID0gcmVSZ2JhSW50ZWdlci5leGVjKGZvcm1hdCkpID8gcmdiYShtWzFdLCBtWzJdLCBtWzNdLCBtWzRdKSAvLyByZ2JhKDI1NSwgMCwgMCwgMSlcbiAgICAgIDogKG0gPSByZVJnYmFQZXJjZW50LmV4ZWMoZm9ybWF0KSkgPyByZ2JhKG1bMV0gKiAyNTUgLyAxMDAsIG1bMl0gKiAyNTUgLyAxMDAsIG1bM10gKiAyNTUgLyAxMDAsIG1bNF0pIC8vIHJnYigxMDAlLCAwJSwgMCUsIDEpXG4gICAgICA6IChtID0gcmVIc2xQZXJjZW50LmV4ZWMoZm9ybWF0KSkgPyBoc2xhKG1bMV0sIG1bMl0gLyAxMDAsIG1bM10gLyAxMDAsIDEpIC8vIGhzbCgxMjAsIDUwJSwgNTAlKVxuICAgICAgOiAobSA9IHJlSHNsYVBlcmNlbnQuZXhlYyhmb3JtYXQpKSA/IGhzbGEobVsxXSwgbVsyXSAvIDEwMCwgbVszXSAvIDEwMCwgbVs0XSkgLy8gaHNsYSgxMjAsIDUwJSwgNTAlLCAxKVxuICAgICAgOiBuYW1lZC5oYXNPd25Qcm9wZXJ0eShmb3JtYXQpID8gcmdibihuYW1lZFtmb3JtYXRdKSAvLyBlc2xpbnQtZGlzYWJsZS1saW5lIG5vLXByb3RvdHlwZS1idWlsdGluc1xuICAgICAgOiBmb3JtYXQgPT09IFwidHJhbnNwYXJlbnRcIiA/IG5ldyBSZ2IoTmFOLCBOYU4sIE5hTiwgMClcbiAgICAgIDogbnVsbDtcbn1cblxuZnVuY3Rpb24gcmdibihuKSB7XG4gIHJldHVybiBuZXcgUmdiKG4gPj4gMTYgJiAweGZmLCBuID4+IDggJiAweGZmLCBuICYgMHhmZiwgMSk7XG59XG5cbmZ1bmN0aW9uIHJnYmEociwgZywgYiwgYSkge1xuICBpZiAoYSA8PSAwKSByID0gZyA9IGIgPSBOYU47XG4gIHJldHVybiBuZXcgUmdiKHIsIGcsIGIsIGEpO1xufVxuXG5leHBvcnQgZnVuY3Rpb24gcmdiQ29udmVydChvKSB7XG4gIGlmICghKG8gaW5zdGFuY2VvZiBDb2xvcikpIG8gPSBjb2xvcihvKTtcbiAgaWYgKCFvKSByZXR1cm4gbmV3IFJnYjtcbiAgbyA9IG8ucmdiKCk7XG4gIHJldHVybiBuZXcgUmdiKG8uciwgby5nLCBvLmIsIG8ub3BhY2l0eSk7XG59XG5cbmV4cG9ydCBmdW5jdGlvbiByZ2IociwgZywgYiwgb3BhY2l0eSkge1xuICByZXR1cm4gYXJndW1lbnRzLmxlbmd0aCA9PT0gMSA/IHJnYkNvbnZlcnQocikgOiBuZXcgUmdiKHIsIGcsIGIsIG9wYWNpdHkgPT0gbnVsbCA/IDEgOiBvcGFjaXR5KTtcbn1cblxuZXhwb3J0IGZ1bmN0aW9uIFJnYihyLCBnLCBiLCBvcGFjaXR5KSB7XG4gIHRoaXMuciA9ICtyO1xuICB0aGlzLmcgPSArZztcbiAgdGhpcy5iID0gK2I7XG4gIHRoaXMub3BhY2l0eSA9ICtvcGFjaXR5O1xufVxuXG5kZWZpbmUoUmdiLCByZ2IsIGV4dGVuZChDb2xvciwge1xuICBicmlnaHRlcihrKSB7XG4gICAgayA9IGsgPT0gbnVsbCA/IGJyaWdodGVyIDogTWF0aC5wb3coYnJpZ2h0ZXIsIGspO1xuICAgIHJldHVybiBuZXcgUmdiKHRoaXMuciAqIGssIHRoaXMuZyAqIGssIHRoaXMuYiAqIGssIHRoaXMub3BhY2l0eSk7XG4gIH0sXG4gIGRhcmtlcihrKSB7XG4gICAgayA9IGsgPT0gbnVsbCA/IGRhcmtlciA6IE1hdGgucG93KGRhcmtlciwgayk7XG4gICAgcmV0dXJuIG5ldyBSZ2IodGhpcy5yICogaywgdGhpcy5nICogaywgdGhpcy5iICogaywgdGhpcy5vcGFjaXR5KTtcbiAgfSxcbiAgcmdiKCkge1xuICAgIHJldHVybiB0aGlzO1xuICB9LFxuICBjbGFtcCgpIHtcbiAgICByZXR1cm4gbmV3IFJnYihjbGFtcGkodGhpcy5yKSwgY2xhbXBpKHRoaXMuZyksIGNsYW1waSh0aGlzLmIpLCBjbGFtcGEodGhpcy5vcGFjaXR5KSk7XG4gIH0sXG4gIGRpc3BsYXlhYmxlKCkge1xuICAgIHJldHVybiAoLTAuNSA8PSB0aGlzLnIgJiYgdGhpcy5yIDwgMjU1LjUpXG4gICAgICAgICYmICgtMC41IDw9IHRoaXMuZyAmJiB0aGlzLmcgPCAyNTUuNSlcbiAgICAgICAgJiYgKC0wLjUgPD0gdGhpcy5iICYmIHRoaXMuYiA8IDI1NS41KVxuICAgICAgICAmJiAoMCA8PSB0aGlzLm9wYWNpdHkgJiYgdGhpcy5vcGFjaXR5IDw9IDEpO1xuICB9LFxuICBoZXg6IHJnYl9mb3JtYXRIZXgsIC8vIERlcHJlY2F0ZWQhIFVzZSBjb2xvci5mb3JtYXRIZXguXG4gIGZvcm1hdEhleDogcmdiX2Zvcm1hdEhleCxcbiAgZm9ybWF0SGV4ODogcmdiX2Zvcm1hdEhleDgsXG4gIGZvcm1hdFJnYjogcmdiX2Zvcm1hdFJnYixcbiAgdG9TdHJpbmc6IHJnYl9mb3JtYXRSZ2Jcbn0pKTtcblxuZnVuY3Rpb24gcmdiX2Zvcm1hdEhleCgpIHtcbiAgcmV0dXJuIGAjJHtoZXgodGhpcy5yKX0ke2hleCh0aGlzLmcpfSR7aGV4KHRoaXMuYil9YDtcbn1cblxuZnVuY3Rpb24gcmdiX2Zvcm1hdEhleDgoKSB7XG4gIHJldHVybiBgIyR7aGV4KHRoaXMucil9JHtoZXgodGhpcy5nKX0ke2hleCh0aGlzLmIpfSR7aGV4KChpc05hTih0aGlzLm9wYWNpdHkpID8gMSA6IHRoaXMub3BhY2l0eSkgKiAyNTUpfWA7XG59XG5cbmZ1bmN0aW9uIHJnYl9mb3JtYXRSZ2IoKSB7XG4gIGNvbnN0IGEgPSBjbGFtcGEodGhpcy5vcGFjaXR5KTtcbiAgcmV0dXJuIGAke2EgPT09IDEgPyBcInJnYihcIiA6IFwicmdiYShcIn0ke2NsYW1waSh0aGlzLnIpfSwgJHtjbGFtcGkodGhpcy5nKX0sICR7Y2xhbXBpKHRoaXMuYil9JHthID09PSAxID8gXCIpXCIgOiBgLCAke2F9KWB9YDtcbn1cblxuZnVuY3Rpb24gY2xhbXBhKG9wYWNpdHkpIHtcbiAgcmV0dXJuIGlzTmFOKG9wYWNpdHkpID8gMSA6IE1hdGgubWF4KDAsIE1hdGgubWluKDEsIG9wYWNpdHkpKTtcbn1cblxuZnVuY3Rpb24gY2xhbXBpKHZhbHVlKSB7XG4gIHJldHVybiBNYXRoLm1heCgwLCBNYXRoLm1pbigyNTUsIE1hdGgucm91bmQodmFsdWUpIHx8IDApKTtcbn1cblxuZnVuY3Rpb24gaGV4KHZhbHVlKSB7XG4gIHZhbHVlID0gY2xhbXBpKHZhbHVlKTtcbiAgcmV0dXJuICh2YWx1ZSA8IDE2ID8gXCIwXCIgOiBcIlwiKSArIHZhbHVlLnRvU3RyaW5nKDE2KTtcbn1cblxuZnVuY3Rpb24gaHNsYShoLCBzLCBsLCBhKSB7XG4gIGlmIChhIDw9IDApIGggPSBzID0gbCA9IE5hTjtcbiAgZWxzZSBpZiAobCA8PSAwIHx8IGwgPj0gMSkgaCA9IHMgPSBOYU47XG4gIGVsc2UgaWYgKHMgPD0gMCkgaCA9IE5hTjtcbiAgcmV0dXJuIG5ldyBIc2woaCwgcywgbCwgYSk7XG59XG5cbmV4cG9ydCBmdW5jdGlvbiBoc2xDb252ZXJ0KG8pIHtcbiAgaWYgKG8gaW5zdGFuY2VvZiBIc2wpIHJldHVybiBuZXcgSHNsKG8uaCwgby5zLCBvLmwsIG8ub3BhY2l0eSk7XG4gIGlmICghKG8gaW5zdGFuY2VvZiBDb2xvcikpIG8gPSBjb2xvcihvKTtcbiAgaWYgKCFvKSByZXR1cm4gbmV3IEhzbDtcbiAgaWYgKG8gaW5zdGFuY2VvZiBIc2wpIHJldHVybiBvO1xuICBvID0gby5yZ2IoKTtcbiAgdmFyIHIgPSBvLnIgLyAyNTUsXG4gICAgICBnID0gby5nIC8gMjU1LFxuICAgICAgYiA9IG8uYiAvIDI1NSxcbiAgICAgIG1pbiA9IE1hdGgubWluKHIsIGcsIGIpLFxuICAgICAgbWF4ID0gTWF0aC5tYXgociwgZywgYiksXG4gICAgICBoID0gTmFOLFxuICAgICAgcyA9IG1heCAtIG1pbixcbiAgICAgIGwgPSAobWF4ICsgbWluKSAvIDI7XG4gIGlmIChzKSB7XG4gICAgaWYgKHIgPT09IG1heCkgaCA9IChnIC0gYikgLyBzICsgKGcgPCBiKSAqIDY7XG4gICAgZWxzZSBpZiAoZyA9PT0gbWF4KSBoID0gKGIgLSByKSAvIHMgKyAyO1xuICAgIGVsc2UgaCA9IChyIC0gZykgLyBzICsgNDtcbiAgICBzIC89IGwgPCAwLjUgPyBtYXggKyBtaW4gOiAyIC0gbWF4IC0gbWluO1xuICAgIGggKj0gNjA7XG4gIH0gZWxzZSB7XG4gICAgcyA9IGwgPiAwICYmIGwgPCAxID8gMCA6IGg7XG4gIH1cbiAgcmV0dXJuIG5ldyBIc2woaCwgcywgbCwgby5vcGFjaXR5KTtcbn1cblxuZXhwb3J0IGZ1bmN0aW9uIGhzbChoLCBzLCBsLCBvcGFjaXR5KSB7XG4gIHJldHVybiBhcmd1bWVudHMubGVuZ3RoID09PSAxID8gaHNsQ29udmVydChoKSA6IG5ldyBIc2woaCwgcywgbCwgb3BhY2l0eSA9PSBudWxsID8gMSA6IG9wYWNpdHkpO1xufVxuXG5mdW5jdGlvbiBIc2woaCwgcywgbCwgb3BhY2l0eSkge1xuICB0aGlzLmggPSAraDtcbiAgdGhpcy5zID0gK3M7XG4gIHRoaXMubCA9ICtsO1xuICB0aGlzLm9wYWNpdHkgPSArb3BhY2l0eTtcbn1cblxuZGVmaW5lKEhzbCwgaHNsLCBleHRlbmQoQ29sb3IsIHtcbiAgYnJpZ2h0ZXIoaykge1xuICAgIGsgPSBrID09IG51bGwgPyBicmlnaHRlciA6IE1hdGgucG93KGJyaWdodGVyLCBrKTtcbiAgICByZXR1cm4gbmV3IEhzbCh0aGlzLmgsIHRoaXMucywgdGhpcy5sICogaywgdGhpcy5vcGFjaXR5KTtcbiAgfSxcbiAgZGFya2VyKGspIHtcbiAgICBrID0gayA9PSBudWxsID8gZGFya2VyIDogTWF0aC5wb3coZGFya2VyLCBrKTtcbiAgICByZXR1cm4gbmV3IEhzbCh0aGlzLmgsIHRoaXMucywgdGhpcy5sICogaywgdGhpcy5vcGFjaXR5KTtcbiAgfSxcbiAgcmdiKCkge1xuICAgIHZhciBoID0gdGhpcy5oICUgMzYwICsgKHRoaXMuaCA8IDApICogMzYwLFxuICAgICAgICBzID0gaXNOYU4oaCkgfHwgaXNOYU4odGhpcy5zKSA/IDAgOiB0aGlzLnMsXG4gICAgICAgIGwgPSB0aGlzLmwsXG4gICAgICAgIG0yID0gbCArIChsIDwgMC41ID8gbCA6IDEgLSBsKSAqIHMsXG4gICAgICAgIG0xID0gMiAqIGwgLSBtMjtcbiAgICByZXR1cm4gbmV3IFJnYihcbiAgICAgIGhzbDJyZ2IoaCA+PSAyNDAgPyBoIC0gMjQwIDogaCArIDEyMCwgbTEsIG0yKSxcbiAgICAgIGhzbDJyZ2IoaCwgbTEsIG0yKSxcbiAgICAgIGhzbDJyZ2IoaCA8IDEyMCA/IGggKyAyNDAgOiBoIC0gMTIwLCBtMSwgbTIpLFxuICAgICAgdGhpcy5vcGFjaXR5XG4gICAgKTtcbiAgfSxcbiAgY2xhbXAoKSB7XG4gICAgcmV0dXJuIG5ldyBIc2woY2xhbXBoKHRoaXMuaCksIGNsYW1wdCh0aGlzLnMpLCBjbGFtcHQodGhpcy5sKSwgY2xhbXBhKHRoaXMub3BhY2l0eSkpO1xuICB9LFxuICBkaXNwbGF5YWJsZSgpIHtcbiAgICByZXR1cm4gKDAgPD0gdGhpcy5zICYmIHRoaXMucyA8PSAxIHx8IGlzTmFOKHRoaXMucykpXG4gICAgICAgICYmICgwIDw9IHRoaXMubCAmJiB0aGlzLmwgPD0gMSlcbiAgICAgICAgJiYgKDAgPD0gdGhpcy5vcGFjaXR5ICYmIHRoaXMub3BhY2l0eSA8PSAxKTtcbiAgfSxcbiAgZm9ybWF0SHNsKCkge1xuICAgIGNvbnN0IGEgPSBjbGFtcGEodGhpcy5vcGFjaXR5KTtcbiAgICByZXR1cm4gYCR7YSA9PT0gMSA/IFwiaHNsKFwiIDogXCJoc2xhKFwifSR7Y2xhbXBoKHRoaXMuaCl9LCAke2NsYW1wdCh0aGlzLnMpICogMTAwfSUsICR7Y2xhbXB0KHRoaXMubCkgKiAxMDB9JSR7YSA9PT0gMSA/IFwiKVwiIDogYCwgJHthfSlgfWA7XG4gIH1cbn0pKTtcblxuZnVuY3Rpb24gY2xhbXBoKHZhbHVlKSB7XG4gIHZhbHVlID0gKHZhbHVlIHx8IDApICUgMzYwO1xuICByZXR1cm4gdmFsdWUgPCAwID8gdmFsdWUgKyAzNjAgOiB2YWx1ZTtcbn1cblxuZnVuY3Rpb24gY2xhbXB0KHZhbHVlKSB7XG4gIHJldHVybiBNYXRoLm1heCgwLCBNYXRoLm1pbigxLCB2YWx1ZSB8fCAwKSk7XG59XG5cbi8qIEZyb20gRnZEIDEzLjM3LCBDU1MgQ29sb3IgTW9kdWxlIExldmVsIDMgKi9cbmZ1bmN0aW9uIGhzbDJyZ2IoaCwgbTEsIG0yKSB7XG4gIHJldHVybiAoaCA8IDYwID8gbTEgKyAobTIgLSBtMSkgKiBoIC8gNjBcbiAgICAgIDogaCA8IDE4MCA/IG0yXG4gICAgICA6IGggPCAyNDAgPyBtMSArIChtMiAtIG0xKSAqICgyNDAgLSBoKSAvIDYwXG4gICAgICA6IG0xKSAqIDI1NTtcbn1cbiIsICJleHBvcnQgZnVuY3Rpb24gYmFzaXModDEsIHYwLCB2MSwgdjIsIHYzKSB7XG4gIHZhciB0MiA9IHQxICogdDEsIHQzID0gdDIgKiB0MTtcbiAgcmV0dXJuICgoMSAtIDMgKiB0MSArIDMgKiB0MiAtIHQzKSAqIHYwXG4gICAgICArICg0IC0gNiAqIHQyICsgMyAqIHQzKSAqIHYxXG4gICAgICArICgxICsgMyAqIHQxICsgMyAqIHQyIC0gMyAqIHQzKSAqIHYyXG4gICAgICArIHQzICogdjMpIC8gNjtcbn1cblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24odmFsdWVzKSB7XG4gIHZhciBuID0gdmFsdWVzLmxlbmd0aCAtIDE7XG4gIHJldHVybiBmdW5jdGlvbih0KSB7XG4gICAgdmFyIGkgPSB0IDw9IDAgPyAodCA9IDApIDogdCA+PSAxID8gKHQgPSAxLCBuIC0gMSkgOiBNYXRoLmZsb29yKHQgKiBuKSxcbiAgICAgICAgdjEgPSB2YWx1ZXNbaV0sXG4gICAgICAgIHYyID0gdmFsdWVzW2kgKyAxXSxcbiAgICAgICAgdjAgPSBpID4gMCA/IHZhbHVlc1tpIC0gMV0gOiAyICogdjEgLSB2MixcbiAgICAgICAgdjMgPSBpIDwgbiAtIDEgPyB2YWx1ZXNbaSArIDJdIDogMiAqIHYyIC0gdjE7XG4gICAgcmV0dXJuIGJhc2lzKCh0IC0gaSAvIG4pICogbiwgdjAsIHYxLCB2MiwgdjMpO1xuICB9O1xufVxuIiwgImltcG9ydCB7YmFzaXN9IGZyb20gXCIuL2Jhc2lzLmpzXCI7XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKHZhbHVlcykge1xuICB2YXIgbiA9IHZhbHVlcy5sZW5ndGg7XG4gIHJldHVybiBmdW5jdGlvbih0KSB7XG4gICAgdmFyIGkgPSBNYXRoLmZsb29yKCgodCAlPSAxKSA8IDAgPyArK3QgOiB0KSAqIG4pLFxuICAgICAgICB2MCA9IHZhbHVlc1soaSArIG4gLSAxKSAlIG5dLFxuICAgICAgICB2MSA9IHZhbHVlc1tpICUgbl0sXG4gICAgICAgIHYyID0gdmFsdWVzWyhpICsgMSkgJSBuXSxcbiAgICAgICAgdjMgPSB2YWx1ZXNbKGkgKyAyKSAlIG5dO1xuICAgIHJldHVybiBiYXNpcygodCAtIGkgLyBuKSAqIG4sIHYwLCB2MSwgdjIsIHYzKTtcbiAgfTtcbn1cbiIsICJleHBvcnQgZGVmYXVsdCB4ID0+ICgpID0+IHg7XG4iLCAiaW1wb3J0IGNvbnN0YW50IGZyb20gXCIuL2NvbnN0YW50LmpzXCI7XG5cbmZ1bmN0aW9uIGxpbmVhcihhLCBkKSB7XG4gIHJldHVybiBmdW5jdGlvbih0KSB7XG4gICAgcmV0dXJuIGEgKyB0ICogZDtcbiAgfTtcbn1cblxuZnVuY3Rpb24gZXhwb25lbnRpYWwoYSwgYiwgeSkge1xuICByZXR1cm4gYSA9IE1hdGgucG93KGEsIHkpLCBiID0gTWF0aC5wb3coYiwgeSkgLSBhLCB5ID0gMSAvIHksIGZ1bmN0aW9uKHQpIHtcbiAgICByZXR1cm4gTWF0aC5wb3coYSArIHQgKiBiLCB5KTtcbiAgfTtcbn1cblxuZXhwb3J0IGZ1bmN0aW9uIGh1ZShhLCBiKSB7XG4gIHZhciBkID0gYiAtIGE7XG4gIHJldHVybiBkID8gbGluZWFyKGEsIGQgPiAxODAgfHwgZCA8IC0xODAgPyBkIC0gMzYwICogTWF0aC5yb3VuZChkIC8gMzYwKSA6IGQpIDogY29uc3RhbnQoaXNOYU4oYSkgPyBiIDogYSk7XG59XG5cbmV4cG9ydCBmdW5jdGlvbiBnYW1tYSh5KSB7XG4gIHJldHVybiAoeSA9ICt5KSA9PT0gMSA/IG5vZ2FtbWEgOiBmdW5jdGlvbihhLCBiKSB7XG4gICAgcmV0dXJuIGIgLSBhID8gZXhwb25lbnRpYWwoYSwgYiwgeSkgOiBjb25zdGFudChpc05hTihhKSA/IGIgOiBhKTtcbiAgfTtcbn1cblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24gbm9nYW1tYShhLCBiKSB7XG4gIHZhciBkID0gYiAtIGE7XG4gIHJldHVybiBkID8gbGluZWFyKGEsIGQpIDogY29uc3RhbnQoaXNOYU4oYSkgPyBiIDogYSk7XG59XG4iLCAiaW1wb3J0IHtyZ2IgYXMgY29sb3JSZ2J9IGZyb20gXCJkMy1jb2xvclwiO1xuaW1wb3J0IGJhc2lzIGZyb20gXCIuL2Jhc2lzLmpzXCI7XG5pbXBvcnQgYmFzaXNDbG9zZWQgZnJvbSBcIi4vYmFzaXNDbG9zZWQuanNcIjtcbmltcG9ydCBub2dhbW1hLCB7Z2FtbWF9IGZyb20gXCIuL2NvbG9yLmpzXCI7XG5cbmV4cG9ydCBkZWZhdWx0IChmdW5jdGlvbiByZ2JHYW1tYSh5KSB7XG4gIHZhciBjb2xvciA9IGdhbW1hKHkpO1xuXG4gIGZ1bmN0aW9uIHJnYihzdGFydCwgZW5kKSB7XG4gICAgdmFyIHIgPSBjb2xvcigoc3RhcnQgPSBjb2xvclJnYihzdGFydCkpLnIsIChlbmQgPSBjb2xvclJnYihlbmQpKS5yKSxcbiAgICAgICAgZyA9IGNvbG9yKHN0YXJ0LmcsIGVuZC5nKSxcbiAgICAgICAgYiA9IGNvbG9yKHN0YXJ0LmIsIGVuZC5iKSxcbiAgICAgICAgb3BhY2l0eSA9IG5vZ2FtbWEoc3RhcnQub3BhY2l0eSwgZW5kLm9wYWNpdHkpO1xuICAgIHJldHVybiBmdW5jdGlvbih0KSB7XG4gICAgICBzdGFydC5yID0gcih0KTtcbiAgICAgIHN0YXJ0LmcgPSBnKHQpO1xuICAgICAgc3RhcnQuYiA9IGIodCk7XG4gICAgICBzdGFydC5vcGFjaXR5ID0gb3BhY2l0eSh0KTtcbiAgICAgIHJldHVybiBzdGFydCArIFwiXCI7XG4gICAgfTtcbiAgfVxuXG4gIHJnYi5nYW1tYSA9IHJnYkdhbW1hO1xuXG4gIHJldHVybiByZ2I7XG59KSgxKTtcblxuZnVuY3Rpb24gcmdiU3BsaW5lKHNwbGluZSkge1xuICByZXR1cm4gZnVuY3Rpb24oY29sb3JzKSB7XG4gICAgdmFyIG4gPSBjb2xvcnMubGVuZ3RoLFxuICAgICAgICByID0gbmV3IEFycmF5KG4pLFxuICAgICAgICBnID0gbmV3IEFycmF5KG4pLFxuICAgICAgICBiID0gbmV3IEFycmF5KG4pLFxuICAgICAgICBpLCBjb2xvcjtcbiAgICBmb3IgKGkgPSAwOyBpIDwgbjsgKytpKSB7XG4gICAgICBjb2xvciA9IGNvbG9yUmdiKGNvbG9yc1tpXSk7XG4gICAgICByW2ldID0gY29sb3IuciB8fCAwO1xuICAgICAgZ1tpXSA9IGNvbG9yLmcgfHwgMDtcbiAgICAgIGJbaV0gPSBjb2xvci5iIHx8IDA7XG4gICAgfVxuICAgIHIgPSBzcGxpbmUocik7XG4gICAgZyA9IHNwbGluZShnKTtcbiAgICBiID0gc3BsaW5lKGIpO1xuICAgIGNvbG9yLm9wYWNpdHkgPSAxO1xuICAgIHJldHVybiBmdW5jdGlvbih0KSB7XG4gICAgICBjb2xvci5yID0gcih0KTtcbiAgICAgIGNvbG9yLmcgPSBnKHQpO1xuICAgICAgY29sb3IuYiA9IGIodCk7XG4gICAgICByZXR1cm4gY29sb3IgKyBcIlwiO1xuICAgIH07XG4gIH07XG59XG5cbmV4cG9ydCB2YXIgcmdiQmFzaXMgPSByZ2JTcGxpbmUoYmFzaXMpO1xuZXhwb3J0IHZhciByZ2JCYXNpc0Nsb3NlZCA9IHJnYlNwbGluZShiYXNpc0Nsb3NlZCk7XG4iLCAiZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24oYSwgYikge1xuICByZXR1cm4gYSA9ICthLCBiID0gK2IsIGZ1bmN0aW9uKHQpIHtcbiAgICByZXR1cm4gYSAqICgxIC0gdCkgKyBiICogdDtcbiAgfTtcbn1cbiIsICJpbXBvcnQgbnVtYmVyIGZyb20gXCIuL251bWJlci5qc1wiO1xuXG52YXIgcmVBID0gL1stK10/KD86XFxkK1xcLj9cXGQqfFxcLj9cXGQrKSg/OltlRV1bLStdP1xcZCspPy9nLFxuICAgIHJlQiA9IG5ldyBSZWdFeHAocmVBLnNvdXJjZSwgXCJnXCIpO1xuXG5mdW5jdGlvbiB6ZXJvKGIpIHtcbiAgcmV0dXJuIGZ1bmN0aW9uKCkge1xuICAgIHJldHVybiBiO1xuICB9O1xufVxuXG5mdW5jdGlvbiBvbmUoYikge1xuICByZXR1cm4gZnVuY3Rpb24odCkge1xuICAgIHJldHVybiBiKHQpICsgXCJcIjtcbiAgfTtcbn1cblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24oYSwgYikge1xuICB2YXIgYmkgPSByZUEubGFzdEluZGV4ID0gcmVCLmxhc3RJbmRleCA9IDAsIC8vIHNjYW4gaW5kZXggZm9yIG5leHQgbnVtYmVyIGluIGJcbiAgICAgIGFtLCAvLyBjdXJyZW50IG1hdGNoIGluIGFcbiAgICAgIGJtLCAvLyBjdXJyZW50IG1hdGNoIGluIGJcbiAgICAgIGJzLCAvLyBzdHJpbmcgcHJlY2VkaW5nIGN1cnJlbnQgbnVtYmVyIGluIGIsIGlmIGFueVxuICAgICAgaSA9IC0xLCAvLyBpbmRleCBpbiBzXG4gICAgICBzID0gW10sIC8vIHN0cmluZyBjb25zdGFudHMgYW5kIHBsYWNlaG9sZGVyc1xuICAgICAgcSA9IFtdOyAvLyBudW1iZXIgaW50ZXJwb2xhdG9yc1xuXG4gIC8vIENvZXJjZSBpbnB1dHMgdG8gc3RyaW5ncy5cbiAgYSA9IGEgKyBcIlwiLCBiID0gYiArIFwiXCI7XG5cbiAgLy8gSW50ZXJwb2xhdGUgcGFpcnMgb2YgbnVtYmVycyBpbiBhICYgYi5cbiAgd2hpbGUgKChhbSA9IHJlQS5leGVjKGEpKVxuICAgICAgJiYgKGJtID0gcmVCLmV4ZWMoYikpKSB7XG4gICAgaWYgKChicyA9IGJtLmluZGV4KSA+IGJpKSB7IC8vIGEgc3RyaW5nIHByZWNlZGVzIHRoZSBuZXh0IG51bWJlciBpbiBiXG4gICAgICBicyA9IGIuc2xpY2UoYmksIGJzKTtcbiAgICAgIGlmIChzW2ldKSBzW2ldICs9IGJzOyAvLyBjb2FsZXNjZSB3aXRoIHByZXZpb3VzIHN0cmluZ1xuICAgICAgZWxzZSBzWysraV0gPSBicztcbiAgICB9XG4gICAgaWYgKChhbSA9IGFtWzBdKSA9PT0gKGJtID0gYm1bMF0pKSB7IC8vIG51bWJlcnMgaW4gYSAmIGIgbWF0Y2hcbiAgICAgIGlmIChzW2ldKSBzW2ldICs9IGJtOyAvLyBjb2FsZXNjZSB3aXRoIHByZXZpb3VzIHN0cmluZ1xuICAgICAgZWxzZSBzWysraV0gPSBibTtcbiAgICB9IGVsc2UgeyAvLyBpbnRlcnBvbGF0ZSBub24tbWF0Y2hpbmcgbnVtYmVyc1xuICAgICAgc1srK2ldID0gbnVsbDtcbiAgICAgIHEucHVzaCh7aTogaSwgeDogbnVtYmVyKGFtLCBibSl9KTtcbiAgICB9XG4gICAgYmkgPSByZUIubGFzdEluZGV4O1xuICB9XG5cbiAgLy8gQWRkIHJlbWFpbnMgb2YgYi5cbiAgaWYgKGJpIDwgYi5sZW5ndGgpIHtcbiAgICBicyA9IGIuc2xpY2UoYmkpO1xuICAgIGlmIChzW2ldKSBzW2ldICs9IGJzOyAvLyBjb2FsZXNjZSB3aXRoIHByZXZpb3VzIHN0cmluZ1xuICAgIGVsc2Ugc1srK2ldID0gYnM7XG4gIH1cblxuICAvLyBTcGVjaWFsIG9wdGltaXphdGlvbiBmb3Igb25seSBhIHNpbmdsZSBtYXRjaC5cbiAgLy8gT3RoZXJ3aXNlLCBpbnRlcnBvbGF0ZSBlYWNoIG9mIHRoZSBudW1iZXJzIGFuZCByZWpvaW4gdGhlIHN0cmluZy5cbiAgcmV0dXJuIHMubGVuZ3RoIDwgMiA/IChxWzBdXG4gICAgICA/IG9uZShxWzBdLngpXG4gICAgICA6IHplcm8oYikpXG4gICAgICA6IChiID0gcS5sZW5ndGgsIGZ1bmN0aW9uKHQpIHtcbiAgICAgICAgICBmb3IgKHZhciBpID0gMCwgbzsgaSA8IGI7ICsraSkgc1sobyA9IHFbaV0pLmldID0gby54KHQpO1xuICAgICAgICAgIHJldHVybiBzLmpvaW4oXCJcIik7XG4gICAgICAgIH0pO1xufVxuIiwgInZhciBkZWdyZWVzID0gMTgwIC8gTWF0aC5QSTtcblxuZXhwb3J0IHZhciBpZGVudGl0eSA9IHtcbiAgdHJhbnNsYXRlWDogMCxcbiAgdHJhbnNsYXRlWTogMCxcbiAgcm90YXRlOiAwLFxuICBza2V3WDogMCxcbiAgc2NhbGVYOiAxLFxuICBzY2FsZVk6IDFcbn07XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKGEsIGIsIGMsIGQsIGUsIGYpIHtcbiAgdmFyIHNjYWxlWCwgc2NhbGVZLCBza2V3WDtcbiAgaWYgKHNjYWxlWCA9IE1hdGguc3FydChhICogYSArIGIgKiBiKSkgYSAvPSBzY2FsZVgsIGIgLz0gc2NhbGVYO1xuICBpZiAoc2tld1ggPSBhICogYyArIGIgKiBkKSBjIC09IGEgKiBza2V3WCwgZCAtPSBiICogc2tld1g7XG4gIGlmIChzY2FsZVkgPSBNYXRoLnNxcnQoYyAqIGMgKyBkICogZCkpIGMgLz0gc2NhbGVZLCBkIC89IHNjYWxlWSwgc2tld1ggLz0gc2NhbGVZO1xuICBpZiAoYSAqIGQgPCBiICogYykgYSA9IC1hLCBiID0gLWIsIHNrZXdYID0gLXNrZXdYLCBzY2FsZVggPSAtc2NhbGVYO1xuICByZXR1cm4ge1xuICAgIHRyYW5zbGF0ZVg6IGUsXG4gICAgdHJhbnNsYXRlWTogZixcbiAgICByb3RhdGU6IE1hdGguYXRhbjIoYiwgYSkgKiBkZWdyZWVzLFxuICAgIHNrZXdYOiBNYXRoLmF0YW4oc2tld1gpICogZGVncmVlcyxcbiAgICBzY2FsZVg6IHNjYWxlWCxcbiAgICBzY2FsZVk6IHNjYWxlWVxuICB9O1xufVxuIiwgImltcG9ydCBkZWNvbXBvc2UsIHtpZGVudGl0eX0gZnJvbSBcIi4vZGVjb21wb3NlLmpzXCI7XG5cbnZhciBzdmdOb2RlO1xuXG4vKiBlc2xpbnQtZGlzYWJsZSBuby11bmRlZiAqL1xuZXhwb3J0IGZ1bmN0aW9uIHBhcnNlQ3NzKHZhbHVlKSB7XG4gIGNvbnN0IG0gPSBuZXcgKHR5cGVvZiBET01NYXRyaXggPT09IFwiZnVuY3Rpb25cIiA/IERPTU1hdHJpeCA6IFdlYktpdENTU01hdHJpeCkodmFsdWUgKyBcIlwiKTtcbiAgcmV0dXJuIG0uaXNJZGVudGl0eSA/IGlkZW50aXR5IDogZGVjb21wb3NlKG0uYSwgbS5iLCBtLmMsIG0uZCwgbS5lLCBtLmYpO1xufVxuXG5leHBvcnQgZnVuY3Rpb24gcGFyc2VTdmcodmFsdWUpIHtcbiAgaWYgKHZhbHVlID09IG51bGwpIHJldHVybiBpZGVudGl0eTtcbiAgaWYgKCFzdmdOb2RlKSBzdmdOb2RlID0gZG9jdW1lbnQuY3JlYXRlRWxlbWVudE5TKFwiaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmdcIiwgXCJnXCIpO1xuICBzdmdOb2RlLnNldEF0dHJpYnV0ZShcInRyYW5zZm9ybVwiLCB2YWx1ZSk7XG4gIGlmICghKHZhbHVlID0gc3ZnTm9kZS50cmFuc2Zvcm0uYmFzZVZhbC5jb25zb2xpZGF0ZSgpKSkgcmV0dXJuIGlkZW50aXR5O1xuICB2YWx1ZSA9IHZhbHVlLm1hdHJpeDtcbiAgcmV0dXJuIGRlY29tcG9zZSh2YWx1ZS5hLCB2YWx1ZS5iLCB2YWx1ZS5jLCB2YWx1ZS5kLCB2YWx1ZS5lLCB2YWx1ZS5mKTtcbn1cbiIsICJpbXBvcnQgbnVtYmVyIGZyb20gXCIuLi9udW1iZXIuanNcIjtcbmltcG9ydCB7cGFyc2VDc3MsIHBhcnNlU3ZnfSBmcm9tIFwiLi9wYXJzZS5qc1wiO1xuXG5mdW5jdGlvbiBpbnRlcnBvbGF0ZVRyYW5zZm9ybShwYXJzZSwgcHhDb21tYSwgcHhQYXJlbiwgZGVnUGFyZW4pIHtcblxuICBmdW5jdGlvbiBwb3Aocykge1xuICAgIHJldHVybiBzLmxlbmd0aCA/IHMucG9wKCkgKyBcIiBcIiA6IFwiXCI7XG4gIH1cblxuICBmdW5jdGlvbiB0cmFuc2xhdGUoeGEsIHlhLCB4YiwgeWIsIHMsIHEpIHtcbiAgICBpZiAoeGEgIT09IHhiIHx8IHlhICE9PSB5Yikge1xuICAgICAgdmFyIGkgPSBzLnB1c2goXCJ0cmFuc2xhdGUoXCIsIG51bGwsIHB4Q29tbWEsIG51bGwsIHB4UGFyZW4pO1xuICAgICAgcS5wdXNoKHtpOiBpIC0gNCwgeDogbnVtYmVyKHhhLCB4Yil9LCB7aTogaSAtIDIsIHg6IG51bWJlcih5YSwgeWIpfSk7XG4gICAgfSBlbHNlIGlmICh4YiB8fCB5Yikge1xuICAgICAgcy5wdXNoKFwidHJhbnNsYXRlKFwiICsgeGIgKyBweENvbW1hICsgeWIgKyBweFBhcmVuKTtcbiAgICB9XG4gIH1cblxuICBmdW5jdGlvbiByb3RhdGUoYSwgYiwgcywgcSkge1xuICAgIGlmIChhICE9PSBiKSB7XG4gICAgICBpZiAoYSAtIGIgPiAxODApIGIgKz0gMzYwOyBlbHNlIGlmIChiIC0gYSA+IDE4MCkgYSArPSAzNjA7IC8vIHNob3J0ZXN0IHBhdGhcbiAgICAgIHEucHVzaCh7aTogcy5wdXNoKHBvcChzKSArIFwicm90YXRlKFwiLCBudWxsLCBkZWdQYXJlbikgLSAyLCB4OiBudW1iZXIoYSwgYil9KTtcbiAgICB9IGVsc2UgaWYgKGIpIHtcbiAgICAgIHMucHVzaChwb3AocykgKyBcInJvdGF0ZShcIiArIGIgKyBkZWdQYXJlbik7XG4gICAgfVxuICB9XG5cbiAgZnVuY3Rpb24gc2tld1goYSwgYiwgcywgcSkge1xuICAgIGlmIChhICE9PSBiKSB7XG4gICAgICBxLnB1c2goe2k6IHMucHVzaChwb3AocykgKyBcInNrZXdYKFwiLCBudWxsLCBkZWdQYXJlbikgLSAyLCB4OiBudW1iZXIoYSwgYil9KTtcbiAgICB9IGVsc2UgaWYgKGIpIHtcbiAgICAgIHMucHVzaChwb3AocykgKyBcInNrZXdYKFwiICsgYiArIGRlZ1BhcmVuKTtcbiAgICB9XG4gIH1cblxuICBmdW5jdGlvbiBzY2FsZSh4YSwgeWEsIHhiLCB5YiwgcywgcSkge1xuICAgIGlmICh4YSAhPT0geGIgfHwgeWEgIT09IHliKSB7XG4gICAgICB2YXIgaSA9IHMucHVzaChwb3AocykgKyBcInNjYWxlKFwiLCBudWxsLCBcIixcIiwgbnVsbCwgXCIpXCIpO1xuICAgICAgcS5wdXNoKHtpOiBpIC0gNCwgeDogbnVtYmVyKHhhLCB4Yil9LCB7aTogaSAtIDIsIHg6IG51bWJlcih5YSwgeWIpfSk7XG4gICAgfSBlbHNlIGlmICh4YiAhPT0gMSB8fCB5YiAhPT0gMSkge1xuICAgICAgcy5wdXNoKHBvcChzKSArIFwic2NhbGUoXCIgKyB4YiArIFwiLFwiICsgeWIgKyBcIilcIik7XG4gICAgfVxuICB9XG5cbiAgcmV0dXJuIGZ1bmN0aW9uKGEsIGIpIHtcbiAgICB2YXIgcyA9IFtdLCAvLyBzdHJpbmcgY29uc3RhbnRzIGFuZCBwbGFjZWhvbGRlcnNcbiAgICAgICAgcSA9IFtdOyAvLyBudW1iZXIgaW50ZXJwb2xhdG9yc1xuICAgIGEgPSBwYXJzZShhKSwgYiA9IHBhcnNlKGIpO1xuICAgIHRyYW5zbGF0ZShhLnRyYW5zbGF0ZVgsIGEudHJhbnNsYXRlWSwgYi50cmFuc2xhdGVYLCBiLnRyYW5zbGF0ZVksIHMsIHEpO1xuICAgIHJvdGF0ZShhLnJvdGF0ZSwgYi5yb3RhdGUsIHMsIHEpO1xuICAgIHNrZXdYKGEuc2tld1gsIGIuc2tld1gsIHMsIHEpO1xuICAgIHNjYWxlKGEuc2NhbGVYLCBhLnNjYWxlWSwgYi5zY2FsZVgsIGIuc2NhbGVZLCBzLCBxKTtcbiAgICBhID0gYiA9IG51bGw7IC8vIGdjXG4gICAgcmV0dXJuIGZ1bmN0aW9uKHQpIHtcbiAgICAgIHZhciBpID0gLTEsIG4gPSBxLmxlbmd0aCwgbztcbiAgICAgIHdoaWxlICgrK2kgPCBuKSBzWyhvID0gcVtpXSkuaV0gPSBvLngodCk7XG4gICAgICByZXR1cm4gcy5qb2luKFwiXCIpO1xuICAgIH07XG4gIH07XG59XG5cbmV4cG9ydCB2YXIgaW50ZXJwb2xhdGVUcmFuc2Zvcm1Dc3MgPSBpbnRlcnBvbGF0ZVRyYW5zZm9ybShwYXJzZUNzcywgXCJweCwgXCIsIFwicHgpXCIsIFwiZGVnKVwiKTtcbmV4cG9ydCB2YXIgaW50ZXJwb2xhdGVUcmFuc2Zvcm1TdmcgPSBpbnRlcnBvbGF0ZVRyYW5zZm9ybShwYXJzZVN2ZywgXCIsIFwiLCBcIilcIiwgXCIpXCIpO1xuIiwgInZhciBmcmFtZSA9IDAsIC8vIGlzIGFuIGFuaW1hdGlvbiBmcmFtZSBwZW5kaW5nP1xuICAgIHRpbWVvdXQgPSAwLCAvLyBpcyBhIHRpbWVvdXQgcGVuZGluZz9cbiAgICBpbnRlcnZhbCA9IDAsIC8vIGFyZSBhbnkgdGltZXJzIGFjdGl2ZT9cbiAgICBwb2tlRGVsYXkgPSAxMDAwLCAvLyBob3cgZnJlcXVlbnRseSB3ZSBjaGVjayBmb3IgY2xvY2sgc2tld1xuICAgIHRhc2tIZWFkLFxuICAgIHRhc2tUYWlsLFxuICAgIGNsb2NrTGFzdCA9IDAsXG4gICAgY2xvY2tOb3cgPSAwLFxuICAgIGNsb2NrU2tldyA9IDAsXG4gICAgY2xvY2sgPSB0eXBlb2YgcGVyZm9ybWFuY2UgPT09IFwib2JqZWN0XCIgJiYgcGVyZm9ybWFuY2Uubm93ID8gcGVyZm9ybWFuY2UgOiBEYXRlLFxuICAgIHNldEZyYW1lID0gdHlwZW9mIHdpbmRvdyA9PT0gXCJvYmplY3RcIiAmJiB3aW5kb3cucmVxdWVzdEFuaW1hdGlvbkZyYW1lID8gd2luZG93LnJlcXVlc3RBbmltYXRpb25GcmFtZS5iaW5kKHdpbmRvdykgOiBmdW5jdGlvbihmKSB7IHNldFRpbWVvdXQoZiwgMTcpOyB9O1xuXG5leHBvcnQgZnVuY3Rpb24gbm93KCkge1xuICByZXR1cm4gY2xvY2tOb3cgfHwgKHNldEZyYW1lKGNsZWFyTm93KSwgY2xvY2tOb3cgPSBjbG9jay5ub3coKSArIGNsb2NrU2tldyk7XG59XG5cbmZ1bmN0aW9uIGNsZWFyTm93KCkge1xuICBjbG9ja05vdyA9IDA7XG59XG5cbmV4cG9ydCBmdW5jdGlvbiBUaW1lcigpIHtcbiAgdGhpcy5fY2FsbCA9XG4gIHRoaXMuX3RpbWUgPVxuICB0aGlzLl9uZXh0ID0gbnVsbDtcbn1cblxuVGltZXIucHJvdG90eXBlID0gdGltZXIucHJvdG90eXBlID0ge1xuICBjb25zdHJ1Y3RvcjogVGltZXIsXG4gIHJlc3RhcnQ6IGZ1bmN0aW9uKGNhbGxiYWNrLCBkZWxheSwgdGltZSkge1xuICAgIGlmICh0eXBlb2YgY2FsbGJhY2sgIT09IFwiZnVuY3Rpb25cIikgdGhyb3cgbmV3IFR5cGVFcnJvcihcImNhbGxiYWNrIGlzIG5vdCBhIGZ1bmN0aW9uXCIpO1xuICAgIHRpbWUgPSAodGltZSA9PSBudWxsID8gbm93KCkgOiArdGltZSkgKyAoZGVsYXkgPT0gbnVsbCA/IDAgOiArZGVsYXkpO1xuICAgIGlmICghdGhpcy5fbmV4dCAmJiB0YXNrVGFpbCAhPT0gdGhpcykge1xuICAgICAgaWYgKHRhc2tUYWlsKSB0YXNrVGFpbC5fbmV4dCA9IHRoaXM7XG4gICAgICBlbHNlIHRhc2tIZWFkID0gdGhpcztcbiAgICAgIHRhc2tUYWlsID0gdGhpcztcbiAgICB9XG4gICAgdGhpcy5fY2FsbCA9IGNhbGxiYWNrO1xuICAgIHRoaXMuX3RpbWUgPSB0aW1lO1xuICAgIHNsZWVwKCk7XG4gIH0sXG4gIHN0b3A6IGZ1bmN0aW9uKCkge1xuICAgIGlmICh0aGlzLl9jYWxsKSB7XG4gICAgICB0aGlzLl9jYWxsID0gbnVsbDtcbiAgICAgIHRoaXMuX3RpbWUgPSBJbmZpbml0eTtcbiAgICAgIHNsZWVwKCk7XG4gICAgfVxuICB9XG59O1xuXG5leHBvcnQgZnVuY3Rpb24gdGltZXIoY2FsbGJhY2ssIGRlbGF5LCB0aW1lKSB7XG4gIHZhciB0ID0gbmV3IFRpbWVyO1xuICB0LnJlc3RhcnQoY2FsbGJhY2ssIGRlbGF5LCB0aW1lKTtcbiAgcmV0dXJuIHQ7XG59XG5cbmV4cG9ydCBmdW5jdGlvbiB0aW1lckZsdXNoKCkge1xuICBub3coKTsgLy8gR2V0IHRoZSBjdXJyZW50IHRpbWUsIGlmIG5vdCBhbHJlYWR5IHNldC5cbiAgKytmcmFtZTsgLy8gUHJldGVuZCB3ZVx1MjAxOXZlIHNldCBhbiBhbGFybSwgaWYgd2UgaGF2ZW5cdTIwMTl0IGFscmVhZHkuXG4gIHZhciB0ID0gdGFza0hlYWQsIGU7XG4gIHdoaWxlICh0KSB7XG4gICAgaWYgKChlID0gY2xvY2tOb3cgLSB0Ll90aW1lKSA+PSAwKSB0Ll9jYWxsLmNhbGwodW5kZWZpbmVkLCBlKTtcbiAgICB0ID0gdC5fbmV4dDtcbiAgfVxuICAtLWZyYW1lO1xufVxuXG5mdW5jdGlvbiB3YWtlKCkge1xuICBjbG9ja05vdyA9IChjbG9ja0xhc3QgPSBjbG9jay5ub3coKSkgKyBjbG9ja1NrZXc7XG4gIGZyYW1lID0gdGltZW91dCA9IDA7XG4gIHRyeSB7XG4gICAgdGltZXJGbHVzaCgpO1xuICB9IGZpbmFsbHkge1xuICAgIGZyYW1lID0gMDtcbiAgICBuYXAoKTtcbiAgICBjbG9ja05vdyA9IDA7XG4gIH1cbn1cblxuZnVuY3Rpb24gcG9rZSgpIHtcbiAgdmFyIG5vdyA9IGNsb2NrLm5vdygpLCBkZWxheSA9IG5vdyAtIGNsb2NrTGFzdDtcbiAgaWYgKGRlbGF5ID4gcG9rZURlbGF5KSBjbG9ja1NrZXcgLT0gZGVsYXksIGNsb2NrTGFzdCA9IG5vdztcbn1cblxuZnVuY3Rpb24gbmFwKCkge1xuICB2YXIgdDAsIHQxID0gdGFza0hlYWQsIHQyLCB0aW1lID0gSW5maW5pdHk7XG4gIHdoaWxlICh0MSkge1xuICAgIGlmICh0MS5fY2FsbCkge1xuICAgICAgaWYgKHRpbWUgPiB0MS5fdGltZSkgdGltZSA9IHQxLl90aW1lO1xuICAgICAgdDAgPSB0MSwgdDEgPSB0MS5fbmV4dDtcbiAgICB9IGVsc2Uge1xuICAgICAgdDIgPSB0MS5fbmV4dCwgdDEuX25leHQgPSBudWxsO1xuICAgICAgdDEgPSB0MCA/IHQwLl9uZXh0ID0gdDIgOiB0YXNrSGVhZCA9IHQyO1xuICAgIH1cbiAgfVxuICB0YXNrVGFpbCA9IHQwO1xuICBzbGVlcCh0aW1lKTtcbn1cblxuZnVuY3Rpb24gc2xlZXAodGltZSkge1xuICBpZiAoZnJhbWUpIHJldHVybjsgLy8gU29vbmVzdCBhbGFybSBhbHJlYWR5IHNldCwgb3Igd2lsbCBiZS5cbiAgaWYgKHRpbWVvdXQpIHRpbWVvdXQgPSBjbGVhclRpbWVvdXQodGltZW91dCk7XG4gIHZhciBkZWxheSA9IHRpbWUgLSBjbG9ja05vdzsgLy8gU3RyaWN0bHkgbGVzcyB0aGFuIGlmIHdlIHJlY29tcHV0ZWQgY2xvY2tOb3cuXG4gIGlmIChkZWxheSA+IDI0KSB7XG4gICAgaWYgKHRpbWUgPCBJbmZpbml0eSkgdGltZW91dCA9IHNldFRpbWVvdXQod2FrZSwgdGltZSAtIGNsb2NrLm5vdygpIC0gY2xvY2tTa2V3KTtcbiAgICBpZiAoaW50ZXJ2YWwpIGludGVydmFsID0gY2xlYXJJbnRlcnZhbChpbnRlcnZhbCk7XG4gIH0gZWxzZSB7XG4gICAgaWYgKCFpbnRlcnZhbCkgY2xvY2tMYXN0ID0gY2xvY2subm93KCksIGludGVydmFsID0gc2V0SW50ZXJ2YWwocG9rZSwgcG9rZURlbGF5KTtcbiAgICBmcmFtZSA9IDEsIHNldEZyYW1lKHdha2UpO1xuICB9XG59XG4iLCAiaW1wb3J0IHtUaW1lcn0gZnJvbSBcIi4vdGltZXIuanNcIjtcblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24oY2FsbGJhY2ssIGRlbGF5LCB0aW1lKSB7XG4gIHZhciB0ID0gbmV3IFRpbWVyO1xuICBkZWxheSA9IGRlbGF5ID09IG51bGwgPyAwIDogK2RlbGF5O1xuICB0LnJlc3RhcnQoZWxhcHNlZCA9PiB7XG4gICAgdC5zdG9wKCk7XG4gICAgY2FsbGJhY2soZWxhcHNlZCArIGRlbGF5KTtcbiAgfSwgZGVsYXksIHRpbWUpO1xuICByZXR1cm4gdDtcbn1cbiIsICJpbXBvcnQge2Rpc3BhdGNofSBmcm9tIFwiZDMtZGlzcGF0Y2hcIjtcbmltcG9ydCB7dGltZXIsIHRpbWVvdXR9IGZyb20gXCJkMy10aW1lclwiO1xuXG52YXIgZW1wdHlPbiA9IGRpc3BhdGNoKFwic3RhcnRcIiwgXCJlbmRcIiwgXCJjYW5jZWxcIiwgXCJpbnRlcnJ1cHRcIik7XG52YXIgZW1wdHlUd2VlbiA9IFtdO1xuXG5leHBvcnQgdmFyIENSRUFURUQgPSAwO1xuZXhwb3J0IHZhciBTQ0hFRFVMRUQgPSAxO1xuZXhwb3J0IHZhciBTVEFSVElORyA9IDI7XG5leHBvcnQgdmFyIFNUQVJURUQgPSAzO1xuZXhwb3J0IHZhciBSVU5OSU5HID0gNDtcbmV4cG9ydCB2YXIgRU5ESU5HID0gNTtcbmV4cG9ydCB2YXIgRU5ERUQgPSA2O1xuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbihub2RlLCBuYW1lLCBpZCwgaW5kZXgsIGdyb3VwLCB0aW1pbmcpIHtcbiAgdmFyIHNjaGVkdWxlcyA9IG5vZGUuX190cmFuc2l0aW9uO1xuICBpZiAoIXNjaGVkdWxlcykgbm9kZS5fX3RyYW5zaXRpb24gPSB7fTtcbiAgZWxzZSBpZiAoaWQgaW4gc2NoZWR1bGVzKSByZXR1cm47XG4gIGNyZWF0ZShub2RlLCBpZCwge1xuICAgIG5hbWU6IG5hbWUsXG4gICAgaW5kZXg6IGluZGV4LCAvLyBGb3IgY29udGV4dCBkdXJpbmcgY2FsbGJhY2suXG4gICAgZ3JvdXA6IGdyb3VwLCAvLyBGb3IgY29udGV4dCBkdXJpbmcgY2FsbGJhY2suXG4gICAgb246IGVtcHR5T24sXG4gICAgdHdlZW46IGVtcHR5VHdlZW4sXG4gICAgdGltZTogdGltaW5nLnRpbWUsXG4gICAgZGVsYXk6IHRpbWluZy5kZWxheSxcbiAgICBkdXJhdGlvbjogdGltaW5nLmR1cmF0aW9uLFxuICAgIGVhc2U6IHRpbWluZy5lYXNlLFxuICAgIHRpbWVyOiBudWxsLFxuICAgIHN0YXRlOiBDUkVBVEVEXG4gIH0pO1xufVxuXG5leHBvcnQgZnVuY3Rpb24gaW5pdChub2RlLCBpZCkge1xuICB2YXIgc2NoZWR1bGUgPSBnZXQobm9kZSwgaWQpO1xuICBpZiAoc2NoZWR1bGUuc3RhdGUgPiBDUkVBVEVEKSB0aHJvdyBuZXcgRXJyb3IoXCJ0b28gbGF0ZTsgYWxyZWFkeSBzY2hlZHVsZWRcIik7XG4gIHJldHVybiBzY2hlZHVsZTtcbn1cblxuZXhwb3J0IGZ1bmN0aW9uIHNldChub2RlLCBpZCkge1xuICB2YXIgc2NoZWR1bGUgPSBnZXQobm9kZSwgaWQpO1xuICBpZiAoc2NoZWR1bGUuc3RhdGUgPiBTVEFSVEVEKSB0aHJvdyBuZXcgRXJyb3IoXCJ0b28gbGF0ZTsgYWxyZWFkeSBydW5uaW5nXCIpO1xuICByZXR1cm4gc2NoZWR1bGU7XG59XG5cbmV4cG9ydCBmdW5jdGlvbiBnZXQobm9kZSwgaWQpIHtcbiAgdmFyIHNjaGVkdWxlID0gbm9kZS5fX3RyYW5zaXRpb247XG4gIGlmICghc2NoZWR1bGUgfHwgIShzY2hlZHVsZSA9IHNjaGVkdWxlW2lkXSkpIHRocm93IG5ldyBFcnJvcihcInRyYW5zaXRpb24gbm90IGZvdW5kXCIpO1xuICByZXR1cm4gc2NoZWR1bGU7XG59XG5cbmZ1bmN0aW9uIGNyZWF0ZShub2RlLCBpZCwgc2VsZikge1xuICB2YXIgc2NoZWR1bGVzID0gbm9kZS5fX3RyYW5zaXRpb24sXG4gICAgICB0d2VlbjtcblxuICAvLyBJbml0aWFsaXplIHRoZSBzZWxmIHRpbWVyIHdoZW4gdGhlIHRyYW5zaXRpb24gaXMgY3JlYXRlZC5cbiAgLy8gTm90ZSB0aGUgYWN0dWFsIGRlbGF5IGlzIG5vdCBrbm93biB1bnRpbCB0aGUgZmlyc3QgY2FsbGJhY2shXG4gIHNjaGVkdWxlc1tpZF0gPSBzZWxmO1xuICBzZWxmLnRpbWVyID0gdGltZXIoc2NoZWR1bGUsIDAsIHNlbGYudGltZSk7XG5cbiAgZnVuY3Rpb24gc2NoZWR1bGUoZWxhcHNlZCkge1xuICAgIHNlbGYuc3RhdGUgPSBTQ0hFRFVMRUQ7XG4gICAgc2VsZi50aW1lci5yZXN0YXJ0KHN0YXJ0LCBzZWxmLmRlbGF5LCBzZWxmLnRpbWUpO1xuXG4gICAgLy8gSWYgdGhlIGVsYXBzZWQgZGVsYXkgaXMgbGVzcyB0aGFuIG91ciBmaXJzdCBzbGVlcCwgc3RhcnQgaW1tZWRpYXRlbHkuXG4gICAgaWYgKHNlbGYuZGVsYXkgPD0gZWxhcHNlZCkgc3RhcnQoZWxhcHNlZCAtIHNlbGYuZGVsYXkpO1xuICB9XG5cbiAgZnVuY3Rpb24gc3RhcnQoZWxhcHNlZCkge1xuICAgIHZhciBpLCBqLCBuLCBvO1xuXG4gICAgLy8gSWYgdGhlIHN0YXRlIGlzIG5vdCBTQ0hFRFVMRUQsIHRoZW4gd2UgcHJldmlvdXNseSBlcnJvcmVkIG9uIHN0YXJ0LlxuICAgIGlmIChzZWxmLnN0YXRlICE9PSBTQ0hFRFVMRUQpIHJldHVybiBzdG9wKCk7XG5cbiAgICBmb3IgKGkgaW4gc2NoZWR1bGVzKSB7XG4gICAgICBvID0gc2NoZWR1bGVzW2ldO1xuICAgICAgaWYgKG8ubmFtZSAhPT0gc2VsZi5uYW1lKSBjb250aW51ZTtcblxuICAgICAgLy8gV2hpbGUgdGhpcyBlbGVtZW50IGFscmVhZHkgaGFzIGEgc3RhcnRpbmcgdHJhbnNpdGlvbiBkdXJpbmcgdGhpcyBmcmFtZSxcbiAgICAgIC8vIGRlZmVyIHN0YXJ0aW5nIGFuIGludGVycnVwdGluZyB0cmFuc2l0aW9uIHVudGlsIHRoYXQgdHJhbnNpdGlvbiBoYXMgYVxuICAgICAgLy8gY2hhbmNlIHRvIHRpY2sgKGFuZCBwb3NzaWJseSBlbmQpOyBzZWUgZDMvZDMtdHJhbnNpdGlvbiM1NCFcbiAgICAgIGlmIChvLnN0YXRlID09PSBTVEFSVEVEKSByZXR1cm4gdGltZW91dChzdGFydCk7XG5cbiAgICAgIC8vIEludGVycnVwdCB0aGUgYWN0aXZlIHRyYW5zaXRpb24sIGlmIGFueS5cbiAgICAgIGlmIChvLnN0YXRlID09PSBSVU5OSU5HKSB7XG4gICAgICAgIG8uc3RhdGUgPSBFTkRFRDtcbiAgICAgICAgby50aW1lci5zdG9wKCk7XG4gICAgICAgIG8ub24uY2FsbChcImludGVycnVwdFwiLCBub2RlLCBub2RlLl9fZGF0YV9fLCBvLmluZGV4LCBvLmdyb3VwKTtcbiAgICAgICAgZGVsZXRlIHNjaGVkdWxlc1tpXTtcbiAgICAgIH1cblxuICAgICAgLy8gQ2FuY2VsIGFueSBwcmUtZW1wdGVkIHRyYW5zaXRpb25zLlxuICAgICAgZWxzZSBpZiAoK2kgPCBpZCkge1xuICAgICAgICBvLnN0YXRlID0gRU5ERUQ7XG4gICAgICAgIG8udGltZXIuc3RvcCgpO1xuICAgICAgICBvLm9uLmNhbGwoXCJjYW5jZWxcIiwgbm9kZSwgbm9kZS5fX2RhdGFfXywgby5pbmRleCwgby5ncm91cCk7XG4gICAgICAgIGRlbGV0ZSBzY2hlZHVsZXNbaV07XG4gICAgICB9XG4gICAgfVxuXG4gICAgLy8gRGVmZXIgdGhlIGZpcnN0IHRpY2sgdG8gZW5kIG9mIHRoZSBjdXJyZW50IGZyYW1lOyBzZWUgZDMvZDMjMTU3Ni5cbiAgICAvLyBOb3RlIHRoZSB0cmFuc2l0aW9uIG1heSBiZSBjYW5jZWxlZCBhZnRlciBzdGFydCBhbmQgYmVmb3JlIHRoZSBmaXJzdCB0aWNrIVxuICAgIC8vIE5vdGUgdGhpcyBtdXN0IGJlIHNjaGVkdWxlZCBiZWZvcmUgdGhlIHN0YXJ0IGV2ZW50OyBzZWUgZDMvZDMtdHJhbnNpdGlvbiMxNiFcbiAgICAvLyBBc3N1bWluZyB0aGlzIGlzIHN1Y2Nlc3NmdWwsIHN1YnNlcXVlbnQgY2FsbGJhY2tzIGdvIHN0cmFpZ2h0IHRvIHRpY2suXG4gICAgdGltZW91dChmdW5jdGlvbigpIHtcbiAgICAgIGlmIChzZWxmLnN0YXRlID09PSBTVEFSVEVEKSB7XG4gICAgICAgIHNlbGYuc3RhdGUgPSBSVU5OSU5HO1xuICAgICAgICBzZWxmLnRpbWVyLnJlc3RhcnQodGljaywgc2VsZi5kZWxheSwgc2VsZi50aW1lKTtcbiAgICAgICAgdGljayhlbGFwc2VkKTtcbiAgICAgIH1cbiAgICB9KTtcblxuICAgIC8vIERpc3BhdGNoIHRoZSBzdGFydCBldmVudC5cbiAgICAvLyBOb3RlIHRoaXMgbXVzdCBiZSBkb25lIGJlZm9yZSB0aGUgdHdlZW4gYXJlIGluaXRpYWxpemVkLlxuICAgIHNlbGYuc3RhdGUgPSBTVEFSVElORztcbiAgICBzZWxmLm9uLmNhbGwoXCJzdGFydFwiLCBub2RlLCBub2RlLl9fZGF0YV9fLCBzZWxmLmluZGV4LCBzZWxmLmdyb3VwKTtcbiAgICBpZiAoc2VsZi5zdGF0ZSAhPT0gU1RBUlRJTkcpIHJldHVybjsgLy8gaW50ZXJydXB0ZWRcbiAgICBzZWxmLnN0YXRlID0gU1RBUlRFRDtcblxuICAgIC8vIEluaXRpYWxpemUgdGhlIHR3ZWVuLCBkZWxldGluZyBudWxsIHR3ZWVuLlxuICAgIHR3ZWVuID0gbmV3IEFycmF5KG4gPSBzZWxmLnR3ZWVuLmxlbmd0aCk7XG4gICAgZm9yIChpID0gMCwgaiA9IC0xOyBpIDwgbjsgKytpKSB7XG4gICAgICBpZiAobyA9IHNlbGYudHdlZW5baV0udmFsdWUuY2FsbChub2RlLCBub2RlLl9fZGF0YV9fLCBzZWxmLmluZGV4LCBzZWxmLmdyb3VwKSkge1xuICAgICAgICB0d2VlblsrK2pdID0gbztcbiAgICAgIH1cbiAgICB9XG4gICAgdHdlZW4ubGVuZ3RoID0gaiArIDE7XG4gIH1cblxuICBmdW5jdGlvbiB0aWNrKGVsYXBzZWQpIHtcbiAgICB2YXIgdCA9IGVsYXBzZWQgPCBzZWxmLmR1cmF0aW9uID8gc2VsZi5lYXNlLmNhbGwobnVsbCwgZWxhcHNlZCAvIHNlbGYuZHVyYXRpb24pIDogKHNlbGYudGltZXIucmVzdGFydChzdG9wKSwgc2VsZi5zdGF0ZSA9IEVORElORywgMSksXG4gICAgICAgIGkgPSAtMSxcbiAgICAgICAgbiA9IHR3ZWVuLmxlbmd0aDtcblxuICAgIHdoaWxlICgrK2kgPCBuKSB7XG4gICAgICB0d2VlbltpXS5jYWxsKG5vZGUsIHQpO1xuICAgIH1cblxuICAgIC8vIERpc3BhdGNoIHRoZSBlbmQgZXZlbnQuXG4gICAgaWYgKHNlbGYuc3RhdGUgPT09IEVORElORykge1xuICAgICAgc2VsZi5vbi5jYWxsKFwiZW5kXCIsIG5vZGUsIG5vZGUuX19kYXRhX18sIHNlbGYuaW5kZXgsIHNlbGYuZ3JvdXApO1xuICAgICAgc3RvcCgpO1xuICAgIH1cbiAgfVxuXG4gIGZ1bmN0aW9uIHN0b3AoKSB7XG4gICAgc2VsZi5zdGF0ZSA9IEVOREVEO1xuICAgIHNlbGYudGltZXIuc3RvcCgpO1xuICAgIGRlbGV0ZSBzY2hlZHVsZXNbaWRdO1xuICAgIGZvciAodmFyIGkgaW4gc2NoZWR1bGVzKSByZXR1cm47IC8vIGVzbGludC1kaXNhYmxlLWxpbmUgbm8tdW51c2VkLXZhcnNcbiAgICBkZWxldGUgbm9kZS5fX3RyYW5zaXRpb247XG4gIH1cbn1cbiIsICJpbXBvcnQge1NUQVJUSU5HLCBFTkRJTkcsIEVOREVEfSBmcm9tIFwiLi90cmFuc2l0aW9uL3NjaGVkdWxlLmpzXCI7XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKG5vZGUsIG5hbWUpIHtcbiAgdmFyIHNjaGVkdWxlcyA9IG5vZGUuX190cmFuc2l0aW9uLFxuICAgICAgc2NoZWR1bGUsXG4gICAgICBhY3RpdmUsXG4gICAgICBlbXB0eSA9IHRydWUsXG4gICAgICBpO1xuXG4gIGlmICghc2NoZWR1bGVzKSByZXR1cm47XG5cbiAgbmFtZSA9IG5hbWUgPT0gbnVsbCA/IG51bGwgOiBuYW1lICsgXCJcIjtcblxuICBmb3IgKGkgaW4gc2NoZWR1bGVzKSB7XG4gICAgaWYgKChzY2hlZHVsZSA9IHNjaGVkdWxlc1tpXSkubmFtZSAhPT0gbmFtZSkgeyBlbXB0eSA9IGZhbHNlOyBjb250aW51ZTsgfVxuICAgIGFjdGl2ZSA9IHNjaGVkdWxlLnN0YXRlID4gU1RBUlRJTkcgJiYgc2NoZWR1bGUuc3RhdGUgPCBFTkRJTkc7XG4gICAgc2NoZWR1bGUuc3RhdGUgPSBFTkRFRDtcbiAgICBzY2hlZHVsZS50aW1lci5zdG9wKCk7XG4gICAgc2NoZWR1bGUub24uY2FsbChhY3RpdmUgPyBcImludGVycnVwdFwiIDogXCJjYW5jZWxcIiwgbm9kZSwgbm9kZS5fX2RhdGFfXywgc2NoZWR1bGUuaW5kZXgsIHNjaGVkdWxlLmdyb3VwKTtcbiAgICBkZWxldGUgc2NoZWR1bGVzW2ldO1xuICB9XG5cbiAgaWYgKGVtcHR5KSBkZWxldGUgbm9kZS5fX3RyYW5zaXRpb247XG59XG4iLCAiaW1wb3J0IGludGVycnVwdCBmcm9tIFwiLi4vaW50ZXJydXB0LmpzXCI7XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKG5hbWUpIHtcbiAgcmV0dXJuIHRoaXMuZWFjaChmdW5jdGlvbigpIHtcbiAgICBpbnRlcnJ1cHQodGhpcywgbmFtZSk7XG4gIH0pO1xufVxuIiwgImltcG9ydCB7Z2V0LCBzZXR9IGZyb20gXCIuL3NjaGVkdWxlLmpzXCI7XG5cbmZ1bmN0aW9uIHR3ZWVuUmVtb3ZlKGlkLCBuYW1lKSB7XG4gIHZhciB0d2VlbjAsIHR3ZWVuMTtcbiAgcmV0dXJuIGZ1bmN0aW9uKCkge1xuICAgIHZhciBzY2hlZHVsZSA9IHNldCh0aGlzLCBpZCksXG4gICAgICAgIHR3ZWVuID0gc2NoZWR1bGUudHdlZW47XG5cbiAgICAvLyBJZiB0aGlzIG5vZGUgc2hhcmVkIHR3ZWVuIHdpdGggdGhlIHByZXZpb3VzIG5vZGUsXG4gICAgLy8ganVzdCBhc3NpZ24gdGhlIHVwZGF0ZWQgc2hhcmVkIHR3ZWVuIGFuZCB3ZVx1MjAxOXJlIGRvbmUhXG4gICAgLy8gT3RoZXJ3aXNlLCBjb3B5LW9uLXdyaXRlLlxuICAgIGlmICh0d2VlbiAhPT0gdHdlZW4wKSB7XG4gICAgICB0d2VlbjEgPSB0d2VlbjAgPSB0d2VlbjtcbiAgICAgIGZvciAodmFyIGkgPSAwLCBuID0gdHdlZW4xLmxlbmd0aDsgaSA8IG47ICsraSkge1xuICAgICAgICBpZiAodHdlZW4xW2ldLm5hbWUgPT09IG5hbWUpIHtcbiAgICAgICAgICB0d2VlbjEgPSB0d2VlbjEuc2xpY2UoKTtcbiAgICAgICAgICB0d2VlbjEuc3BsaWNlKGksIDEpO1xuICAgICAgICAgIGJyZWFrO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuXG4gICAgc2NoZWR1bGUudHdlZW4gPSB0d2VlbjE7XG4gIH07XG59XG5cbmZ1bmN0aW9uIHR3ZWVuRnVuY3Rpb24oaWQsIG5hbWUsIHZhbHVlKSB7XG4gIHZhciB0d2VlbjAsIHR3ZWVuMTtcbiAgaWYgKHR5cGVvZiB2YWx1ZSAhPT0gXCJmdW5jdGlvblwiKSB0aHJvdyBuZXcgRXJyb3I7XG4gIHJldHVybiBmdW5jdGlvbigpIHtcbiAgICB2YXIgc2NoZWR1bGUgPSBzZXQodGhpcywgaWQpLFxuICAgICAgICB0d2VlbiA9IHNjaGVkdWxlLnR3ZWVuO1xuXG4gICAgLy8gSWYgdGhpcyBub2RlIHNoYXJlZCB0d2VlbiB3aXRoIHRoZSBwcmV2aW91cyBub2RlLFxuICAgIC8vIGp1c3QgYXNzaWduIHRoZSB1cGRhdGVkIHNoYXJlZCB0d2VlbiBhbmQgd2VcdTIwMTlyZSBkb25lIVxuICAgIC8vIE90aGVyd2lzZSwgY29weS1vbi13cml0ZS5cbiAgICBpZiAodHdlZW4gIT09IHR3ZWVuMCkge1xuICAgICAgdHdlZW4xID0gKHR3ZWVuMCA9IHR3ZWVuKS5zbGljZSgpO1xuICAgICAgZm9yICh2YXIgdCA9IHtuYW1lOiBuYW1lLCB2YWx1ZTogdmFsdWV9LCBpID0gMCwgbiA9IHR3ZWVuMS5sZW5ndGg7IGkgPCBuOyArK2kpIHtcbiAgICAgICAgaWYgKHR3ZWVuMVtpXS5uYW1lID09PSBuYW1lKSB7XG4gICAgICAgICAgdHdlZW4xW2ldID0gdDtcbiAgICAgICAgICBicmVhaztcbiAgICAgICAgfVxuICAgICAgfVxuICAgICAgaWYgKGkgPT09IG4pIHR3ZWVuMS5wdXNoKHQpO1xuICAgIH1cblxuICAgIHNjaGVkdWxlLnR3ZWVuID0gdHdlZW4xO1xuICB9O1xufVxuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbihuYW1lLCB2YWx1ZSkge1xuICB2YXIgaWQgPSB0aGlzLl9pZDtcblxuICBuYW1lICs9IFwiXCI7XG5cbiAgaWYgKGFyZ3VtZW50cy5sZW5ndGggPCAyKSB7XG4gICAgdmFyIHR3ZWVuID0gZ2V0KHRoaXMubm9kZSgpLCBpZCkudHdlZW47XG4gICAgZm9yICh2YXIgaSA9IDAsIG4gPSB0d2Vlbi5sZW5ndGgsIHQ7IGkgPCBuOyArK2kpIHtcbiAgICAgIGlmICgodCA9IHR3ZWVuW2ldKS5uYW1lID09PSBuYW1lKSB7XG4gICAgICAgIHJldHVybiB0LnZhbHVlO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gbnVsbDtcbiAgfVxuXG4gIHJldHVybiB0aGlzLmVhY2goKHZhbHVlID09IG51bGwgPyB0d2VlblJlbW92ZSA6IHR3ZWVuRnVuY3Rpb24pKGlkLCBuYW1lLCB2YWx1ZSkpO1xufVxuXG5leHBvcnQgZnVuY3Rpb24gdHdlZW5WYWx1ZSh0cmFuc2l0aW9uLCBuYW1lLCB2YWx1ZSkge1xuICB2YXIgaWQgPSB0cmFuc2l0aW9uLl9pZDtcblxuICB0cmFuc2l0aW9uLmVhY2goZnVuY3Rpb24oKSB7XG4gICAgdmFyIHNjaGVkdWxlID0gc2V0KHRoaXMsIGlkKTtcbiAgICAoc2NoZWR1bGUudmFsdWUgfHwgKHNjaGVkdWxlLnZhbHVlID0ge30pKVtuYW1lXSA9IHZhbHVlLmFwcGx5KHRoaXMsIGFyZ3VtZW50cyk7XG4gIH0pO1xuXG4gIHJldHVybiBmdW5jdGlvbihub2RlKSB7XG4gICAgcmV0dXJuIGdldChub2RlLCBpZCkudmFsdWVbbmFtZV07XG4gIH07XG59XG4iLCAiaW1wb3J0IHtjb2xvcn0gZnJvbSBcImQzLWNvbG9yXCI7XG5pbXBvcnQge2ludGVycG9sYXRlTnVtYmVyLCBpbnRlcnBvbGF0ZVJnYiwgaW50ZXJwb2xhdGVTdHJpbmd9IGZyb20gXCJkMy1pbnRlcnBvbGF0ZVwiO1xuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbihhLCBiKSB7XG4gIHZhciBjO1xuICByZXR1cm4gKHR5cGVvZiBiID09PSBcIm51bWJlclwiID8gaW50ZXJwb2xhdGVOdW1iZXJcbiAgICAgIDogYiBpbnN0YW5jZW9mIGNvbG9yID8gaW50ZXJwb2xhdGVSZ2JcbiAgICAgIDogKGMgPSBjb2xvcihiKSkgPyAoYiA9IGMsIGludGVycG9sYXRlUmdiKVxuICAgICAgOiBpbnRlcnBvbGF0ZVN0cmluZykoYSwgYik7XG59XG4iLCAiaW1wb3J0IHtpbnRlcnBvbGF0ZVRyYW5zZm9ybVN2ZyBhcyBpbnRlcnBvbGF0ZVRyYW5zZm9ybX0gZnJvbSBcImQzLWludGVycG9sYXRlXCI7XG5pbXBvcnQge25hbWVzcGFjZX0gZnJvbSBcImQzLXNlbGVjdGlvblwiO1xuaW1wb3J0IHt0d2VlblZhbHVlfSBmcm9tIFwiLi90d2Vlbi5qc1wiO1xuaW1wb3J0IGludGVycG9sYXRlIGZyb20gXCIuL2ludGVycG9sYXRlLmpzXCI7XG5cbmZ1bmN0aW9uIGF0dHJSZW1vdmUobmFtZSkge1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgdGhpcy5yZW1vdmVBdHRyaWJ1dGUobmFtZSk7XG4gIH07XG59XG5cbmZ1bmN0aW9uIGF0dHJSZW1vdmVOUyhmdWxsbmFtZSkge1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgdGhpcy5yZW1vdmVBdHRyaWJ1dGVOUyhmdWxsbmFtZS5zcGFjZSwgZnVsbG5hbWUubG9jYWwpO1xuICB9O1xufVxuXG5mdW5jdGlvbiBhdHRyQ29uc3RhbnQobmFtZSwgaW50ZXJwb2xhdGUsIHZhbHVlMSkge1xuICB2YXIgc3RyaW5nMDAsXG4gICAgICBzdHJpbmcxID0gdmFsdWUxICsgXCJcIixcbiAgICAgIGludGVycG9sYXRlMDtcbiAgcmV0dXJuIGZ1bmN0aW9uKCkge1xuICAgIHZhciBzdHJpbmcwID0gdGhpcy5nZXRBdHRyaWJ1dGUobmFtZSk7XG4gICAgcmV0dXJuIHN0cmluZzAgPT09IHN0cmluZzEgPyBudWxsXG4gICAgICAgIDogc3RyaW5nMCA9PT0gc3RyaW5nMDAgPyBpbnRlcnBvbGF0ZTBcbiAgICAgICAgOiBpbnRlcnBvbGF0ZTAgPSBpbnRlcnBvbGF0ZShzdHJpbmcwMCA9IHN0cmluZzAsIHZhbHVlMSk7XG4gIH07XG59XG5cbmZ1bmN0aW9uIGF0dHJDb25zdGFudE5TKGZ1bGxuYW1lLCBpbnRlcnBvbGF0ZSwgdmFsdWUxKSB7XG4gIHZhciBzdHJpbmcwMCxcbiAgICAgIHN0cmluZzEgPSB2YWx1ZTEgKyBcIlwiLFxuICAgICAgaW50ZXJwb2xhdGUwO1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgdmFyIHN0cmluZzAgPSB0aGlzLmdldEF0dHJpYnV0ZU5TKGZ1bGxuYW1lLnNwYWNlLCBmdWxsbmFtZS5sb2NhbCk7XG4gICAgcmV0dXJuIHN0cmluZzAgPT09IHN0cmluZzEgPyBudWxsXG4gICAgICAgIDogc3RyaW5nMCA9PT0gc3RyaW5nMDAgPyBpbnRlcnBvbGF0ZTBcbiAgICAgICAgOiBpbnRlcnBvbGF0ZTAgPSBpbnRlcnBvbGF0ZShzdHJpbmcwMCA9IHN0cmluZzAsIHZhbHVlMSk7XG4gIH07XG59XG5cbmZ1bmN0aW9uIGF0dHJGdW5jdGlvbihuYW1lLCBpbnRlcnBvbGF0ZSwgdmFsdWUpIHtcbiAgdmFyIHN0cmluZzAwLFxuICAgICAgc3RyaW5nMTAsXG4gICAgICBpbnRlcnBvbGF0ZTA7XG4gIHJldHVybiBmdW5jdGlvbigpIHtcbiAgICB2YXIgc3RyaW5nMCwgdmFsdWUxID0gdmFsdWUodGhpcyksIHN0cmluZzE7XG4gICAgaWYgKHZhbHVlMSA9PSBudWxsKSByZXR1cm4gdm9pZCB0aGlzLnJlbW92ZUF0dHJpYnV0ZShuYW1lKTtcbiAgICBzdHJpbmcwID0gdGhpcy5nZXRBdHRyaWJ1dGUobmFtZSk7XG4gICAgc3RyaW5nMSA9IHZhbHVlMSArIFwiXCI7XG4gICAgcmV0dXJuIHN0cmluZzAgPT09IHN0cmluZzEgPyBudWxsXG4gICAgICAgIDogc3RyaW5nMCA9PT0gc3RyaW5nMDAgJiYgc3RyaW5nMSA9PT0gc3RyaW5nMTAgPyBpbnRlcnBvbGF0ZTBcbiAgICAgICAgOiAoc3RyaW5nMTAgPSBzdHJpbmcxLCBpbnRlcnBvbGF0ZTAgPSBpbnRlcnBvbGF0ZShzdHJpbmcwMCA9IHN0cmluZzAsIHZhbHVlMSkpO1xuICB9O1xufVxuXG5mdW5jdGlvbiBhdHRyRnVuY3Rpb25OUyhmdWxsbmFtZSwgaW50ZXJwb2xhdGUsIHZhbHVlKSB7XG4gIHZhciBzdHJpbmcwMCxcbiAgICAgIHN0cmluZzEwLFxuICAgICAgaW50ZXJwb2xhdGUwO1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgdmFyIHN0cmluZzAsIHZhbHVlMSA9IHZhbHVlKHRoaXMpLCBzdHJpbmcxO1xuICAgIGlmICh2YWx1ZTEgPT0gbnVsbCkgcmV0dXJuIHZvaWQgdGhpcy5yZW1vdmVBdHRyaWJ1dGVOUyhmdWxsbmFtZS5zcGFjZSwgZnVsbG5hbWUubG9jYWwpO1xuICAgIHN0cmluZzAgPSB0aGlzLmdldEF0dHJpYnV0ZU5TKGZ1bGxuYW1lLnNwYWNlLCBmdWxsbmFtZS5sb2NhbCk7XG4gICAgc3RyaW5nMSA9IHZhbHVlMSArIFwiXCI7XG4gICAgcmV0dXJuIHN0cmluZzAgPT09IHN0cmluZzEgPyBudWxsXG4gICAgICAgIDogc3RyaW5nMCA9PT0gc3RyaW5nMDAgJiYgc3RyaW5nMSA9PT0gc3RyaW5nMTAgPyBpbnRlcnBvbGF0ZTBcbiAgICAgICAgOiAoc3RyaW5nMTAgPSBzdHJpbmcxLCBpbnRlcnBvbGF0ZTAgPSBpbnRlcnBvbGF0ZShzdHJpbmcwMCA9IHN0cmluZzAsIHZhbHVlMSkpO1xuICB9O1xufVxuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbihuYW1lLCB2YWx1ZSkge1xuICB2YXIgZnVsbG5hbWUgPSBuYW1lc3BhY2UobmFtZSksIGkgPSBmdWxsbmFtZSA9PT0gXCJ0cmFuc2Zvcm1cIiA/IGludGVycG9sYXRlVHJhbnNmb3JtIDogaW50ZXJwb2xhdGU7XG4gIHJldHVybiB0aGlzLmF0dHJUd2VlbihuYW1lLCB0eXBlb2YgdmFsdWUgPT09IFwiZnVuY3Rpb25cIlxuICAgICAgPyAoZnVsbG5hbWUubG9jYWwgPyBhdHRyRnVuY3Rpb25OUyA6IGF0dHJGdW5jdGlvbikoZnVsbG5hbWUsIGksIHR3ZWVuVmFsdWUodGhpcywgXCJhdHRyLlwiICsgbmFtZSwgdmFsdWUpKVxuICAgICAgOiB2YWx1ZSA9PSBudWxsID8gKGZ1bGxuYW1lLmxvY2FsID8gYXR0clJlbW92ZU5TIDogYXR0clJlbW92ZSkoZnVsbG5hbWUpXG4gICAgICA6IChmdWxsbmFtZS5sb2NhbCA/IGF0dHJDb25zdGFudE5TIDogYXR0ckNvbnN0YW50KShmdWxsbmFtZSwgaSwgdmFsdWUpKTtcbn1cbiIsICJpbXBvcnQge25hbWVzcGFjZX0gZnJvbSBcImQzLXNlbGVjdGlvblwiO1xuXG5mdW5jdGlvbiBhdHRySW50ZXJwb2xhdGUobmFtZSwgaSkge1xuICByZXR1cm4gZnVuY3Rpb24odCkge1xuICAgIHRoaXMuc2V0QXR0cmlidXRlKG5hbWUsIGkuY2FsbCh0aGlzLCB0KSk7XG4gIH07XG59XG5cbmZ1bmN0aW9uIGF0dHJJbnRlcnBvbGF0ZU5TKGZ1bGxuYW1lLCBpKSB7XG4gIHJldHVybiBmdW5jdGlvbih0KSB7XG4gICAgdGhpcy5zZXRBdHRyaWJ1dGVOUyhmdWxsbmFtZS5zcGFjZSwgZnVsbG5hbWUubG9jYWwsIGkuY2FsbCh0aGlzLCB0KSk7XG4gIH07XG59XG5cbmZ1bmN0aW9uIGF0dHJUd2Vlbk5TKGZ1bGxuYW1lLCB2YWx1ZSkge1xuICB2YXIgdDAsIGkwO1xuICBmdW5jdGlvbiB0d2VlbigpIHtcbiAgICB2YXIgaSA9IHZhbHVlLmFwcGx5KHRoaXMsIGFyZ3VtZW50cyk7XG4gICAgaWYgKGkgIT09IGkwKSB0MCA9IChpMCA9IGkpICYmIGF0dHJJbnRlcnBvbGF0ZU5TKGZ1bGxuYW1lLCBpKTtcbiAgICByZXR1cm4gdDA7XG4gIH1cbiAgdHdlZW4uX3ZhbHVlID0gdmFsdWU7XG4gIHJldHVybiB0d2Vlbjtcbn1cblxuZnVuY3Rpb24gYXR0clR3ZWVuKG5hbWUsIHZhbHVlKSB7XG4gIHZhciB0MCwgaTA7XG4gIGZ1bmN0aW9uIHR3ZWVuKCkge1xuICAgIHZhciBpID0gdmFsdWUuYXBwbHkodGhpcywgYXJndW1lbnRzKTtcbiAgICBpZiAoaSAhPT0gaTApIHQwID0gKGkwID0gaSkgJiYgYXR0ckludGVycG9sYXRlKG5hbWUsIGkpO1xuICAgIHJldHVybiB0MDtcbiAgfVxuICB0d2Vlbi5fdmFsdWUgPSB2YWx1ZTtcbiAgcmV0dXJuIHR3ZWVuO1xufVxuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbihuYW1lLCB2YWx1ZSkge1xuICB2YXIga2V5ID0gXCJhdHRyLlwiICsgbmFtZTtcbiAgaWYgKGFyZ3VtZW50cy5sZW5ndGggPCAyKSByZXR1cm4gKGtleSA9IHRoaXMudHdlZW4oa2V5KSkgJiYga2V5Ll92YWx1ZTtcbiAgaWYgKHZhbHVlID09IG51bGwpIHJldHVybiB0aGlzLnR3ZWVuKGtleSwgbnVsbCk7XG4gIGlmICh0eXBlb2YgdmFsdWUgIT09IFwiZnVuY3Rpb25cIikgdGhyb3cgbmV3IEVycm9yO1xuICB2YXIgZnVsbG5hbWUgPSBuYW1lc3BhY2UobmFtZSk7XG4gIHJldHVybiB0aGlzLnR3ZWVuKGtleSwgKGZ1bGxuYW1lLmxvY2FsID8gYXR0clR3ZWVuTlMgOiBhdHRyVHdlZW4pKGZ1bGxuYW1lLCB2YWx1ZSkpO1xufVxuIiwgImltcG9ydCB7Z2V0LCBpbml0fSBmcm9tIFwiLi9zY2hlZHVsZS5qc1wiO1xuXG5mdW5jdGlvbiBkZWxheUZ1bmN0aW9uKGlkLCB2YWx1ZSkge1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgaW5pdCh0aGlzLCBpZCkuZGVsYXkgPSArdmFsdWUuYXBwbHkodGhpcywgYXJndW1lbnRzKTtcbiAgfTtcbn1cblxuZnVuY3Rpb24gZGVsYXlDb25zdGFudChpZCwgdmFsdWUpIHtcbiAgcmV0dXJuIHZhbHVlID0gK3ZhbHVlLCBmdW5jdGlvbigpIHtcbiAgICBpbml0KHRoaXMsIGlkKS5kZWxheSA9IHZhbHVlO1xuICB9O1xufVxuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbih2YWx1ZSkge1xuICB2YXIgaWQgPSB0aGlzLl9pZDtcblxuICByZXR1cm4gYXJndW1lbnRzLmxlbmd0aFxuICAgICAgPyB0aGlzLmVhY2goKHR5cGVvZiB2YWx1ZSA9PT0gXCJmdW5jdGlvblwiXG4gICAgICAgICAgPyBkZWxheUZ1bmN0aW9uXG4gICAgICAgICAgOiBkZWxheUNvbnN0YW50KShpZCwgdmFsdWUpKVxuICAgICAgOiBnZXQodGhpcy5ub2RlKCksIGlkKS5kZWxheTtcbn1cbiIsICJpbXBvcnQge2dldCwgc2V0fSBmcm9tIFwiLi9zY2hlZHVsZS5qc1wiO1xuXG5mdW5jdGlvbiBkdXJhdGlvbkZ1bmN0aW9uKGlkLCB2YWx1ZSkge1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgc2V0KHRoaXMsIGlkKS5kdXJhdGlvbiA9ICt2YWx1ZS5hcHBseSh0aGlzLCBhcmd1bWVudHMpO1xuICB9O1xufVxuXG5mdW5jdGlvbiBkdXJhdGlvbkNvbnN0YW50KGlkLCB2YWx1ZSkge1xuICByZXR1cm4gdmFsdWUgPSArdmFsdWUsIGZ1bmN0aW9uKCkge1xuICAgIHNldCh0aGlzLCBpZCkuZHVyYXRpb24gPSB2YWx1ZTtcbiAgfTtcbn1cblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24odmFsdWUpIHtcbiAgdmFyIGlkID0gdGhpcy5faWQ7XG5cbiAgcmV0dXJuIGFyZ3VtZW50cy5sZW5ndGhcbiAgICAgID8gdGhpcy5lYWNoKCh0eXBlb2YgdmFsdWUgPT09IFwiZnVuY3Rpb25cIlxuICAgICAgICAgID8gZHVyYXRpb25GdW5jdGlvblxuICAgICAgICAgIDogZHVyYXRpb25Db25zdGFudCkoaWQsIHZhbHVlKSlcbiAgICAgIDogZ2V0KHRoaXMubm9kZSgpLCBpZCkuZHVyYXRpb247XG59XG4iLCAiaW1wb3J0IHtnZXQsIHNldH0gZnJvbSBcIi4vc2NoZWR1bGUuanNcIjtcblxuZnVuY3Rpb24gZWFzZUNvbnN0YW50KGlkLCB2YWx1ZSkge1xuICBpZiAodHlwZW9mIHZhbHVlICE9PSBcImZ1bmN0aW9uXCIpIHRocm93IG5ldyBFcnJvcjtcbiAgcmV0dXJuIGZ1bmN0aW9uKCkge1xuICAgIHNldCh0aGlzLCBpZCkuZWFzZSA9IHZhbHVlO1xuICB9O1xufVxuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbih2YWx1ZSkge1xuICB2YXIgaWQgPSB0aGlzLl9pZDtcblxuICByZXR1cm4gYXJndW1lbnRzLmxlbmd0aFxuICAgICAgPyB0aGlzLmVhY2goZWFzZUNvbnN0YW50KGlkLCB2YWx1ZSkpXG4gICAgICA6IGdldCh0aGlzLm5vZGUoKSwgaWQpLmVhc2U7XG59XG4iLCAiaW1wb3J0IHtzZXR9IGZyb20gXCIuL3NjaGVkdWxlLmpzXCI7XG5cbmZ1bmN0aW9uIGVhc2VWYXJ5aW5nKGlkLCB2YWx1ZSkge1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgdmFyIHYgPSB2YWx1ZS5hcHBseSh0aGlzLCBhcmd1bWVudHMpO1xuICAgIGlmICh0eXBlb2YgdiAhPT0gXCJmdW5jdGlvblwiKSB0aHJvdyBuZXcgRXJyb3I7XG4gICAgc2V0KHRoaXMsIGlkKS5lYXNlID0gdjtcbiAgfTtcbn1cblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24odmFsdWUpIHtcbiAgaWYgKHR5cGVvZiB2YWx1ZSAhPT0gXCJmdW5jdGlvblwiKSB0aHJvdyBuZXcgRXJyb3I7XG4gIHJldHVybiB0aGlzLmVhY2goZWFzZVZhcnlpbmcodGhpcy5faWQsIHZhbHVlKSk7XG59XG4iLCAiaW1wb3J0IHttYXRjaGVyfSBmcm9tIFwiZDMtc2VsZWN0aW9uXCI7XG5pbXBvcnQge1RyYW5zaXRpb259IGZyb20gXCIuL2luZGV4LmpzXCI7XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKG1hdGNoKSB7XG4gIGlmICh0eXBlb2YgbWF0Y2ggIT09IFwiZnVuY3Rpb25cIikgbWF0Y2ggPSBtYXRjaGVyKG1hdGNoKTtcblxuICBmb3IgKHZhciBncm91cHMgPSB0aGlzLl9ncm91cHMsIG0gPSBncm91cHMubGVuZ3RoLCBzdWJncm91cHMgPSBuZXcgQXJyYXkobSksIGogPSAwOyBqIDwgbTsgKytqKSB7XG4gICAgZm9yICh2YXIgZ3JvdXAgPSBncm91cHNbal0sIG4gPSBncm91cC5sZW5ndGgsIHN1Ymdyb3VwID0gc3ViZ3JvdXBzW2pdID0gW10sIG5vZGUsIGkgPSAwOyBpIDwgbjsgKytpKSB7XG4gICAgICBpZiAoKG5vZGUgPSBncm91cFtpXSkgJiYgbWF0Y2guY2FsbChub2RlLCBub2RlLl9fZGF0YV9fLCBpLCBncm91cCkpIHtcbiAgICAgICAgc3ViZ3JvdXAucHVzaChub2RlKTtcbiAgICAgIH1cbiAgICB9XG4gIH1cblxuICByZXR1cm4gbmV3IFRyYW5zaXRpb24oc3ViZ3JvdXBzLCB0aGlzLl9wYXJlbnRzLCB0aGlzLl9uYW1lLCB0aGlzLl9pZCk7XG59XG4iLCAiaW1wb3J0IHtUcmFuc2l0aW9ufSBmcm9tIFwiLi9pbmRleC5qc1wiO1xuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbih0cmFuc2l0aW9uKSB7XG4gIGlmICh0cmFuc2l0aW9uLl9pZCAhPT0gdGhpcy5faWQpIHRocm93IG5ldyBFcnJvcjtcblxuICBmb3IgKHZhciBncm91cHMwID0gdGhpcy5fZ3JvdXBzLCBncm91cHMxID0gdHJhbnNpdGlvbi5fZ3JvdXBzLCBtMCA9IGdyb3VwczAubGVuZ3RoLCBtMSA9IGdyb3VwczEubGVuZ3RoLCBtID0gTWF0aC5taW4obTAsIG0xKSwgbWVyZ2VzID0gbmV3IEFycmF5KG0wKSwgaiA9IDA7IGogPCBtOyArK2opIHtcbiAgICBmb3IgKHZhciBncm91cDAgPSBncm91cHMwW2pdLCBncm91cDEgPSBncm91cHMxW2pdLCBuID0gZ3JvdXAwLmxlbmd0aCwgbWVyZ2UgPSBtZXJnZXNbal0gPSBuZXcgQXJyYXkobiksIG5vZGUsIGkgPSAwOyBpIDwgbjsgKytpKSB7XG4gICAgICBpZiAobm9kZSA9IGdyb3VwMFtpXSB8fCBncm91cDFbaV0pIHtcbiAgICAgICAgbWVyZ2VbaV0gPSBub2RlO1xuICAgICAgfVxuICAgIH1cbiAgfVxuXG4gIGZvciAoOyBqIDwgbTA7ICsraikge1xuICAgIG1lcmdlc1tqXSA9IGdyb3VwczBbal07XG4gIH1cblxuICByZXR1cm4gbmV3IFRyYW5zaXRpb24obWVyZ2VzLCB0aGlzLl9wYXJlbnRzLCB0aGlzLl9uYW1lLCB0aGlzLl9pZCk7XG59XG4iLCAiaW1wb3J0IHtnZXQsIHNldCwgaW5pdH0gZnJvbSBcIi4vc2NoZWR1bGUuanNcIjtcblxuZnVuY3Rpb24gc3RhcnQobmFtZSkge1xuICByZXR1cm4gKG5hbWUgKyBcIlwiKS50cmltKCkuc3BsaXQoL158XFxzKy8pLmV2ZXJ5KGZ1bmN0aW9uKHQpIHtcbiAgICB2YXIgaSA9IHQuaW5kZXhPZihcIi5cIik7XG4gICAgaWYgKGkgPj0gMCkgdCA9IHQuc2xpY2UoMCwgaSk7XG4gICAgcmV0dXJuICF0IHx8IHQgPT09IFwic3RhcnRcIjtcbiAgfSk7XG59XG5cbmZ1bmN0aW9uIG9uRnVuY3Rpb24oaWQsIG5hbWUsIGxpc3RlbmVyKSB7XG4gIHZhciBvbjAsIG9uMSwgc2l0ID0gc3RhcnQobmFtZSkgPyBpbml0IDogc2V0O1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgdmFyIHNjaGVkdWxlID0gc2l0KHRoaXMsIGlkKSxcbiAgICAgICAgb24gPSBzY2hlZHVsZS5vbjtcblxuICAgIC8vIElmIHRoaXMgbm9kZSBzaGFyZWQgYSBkaXNwYXRjaCB3aXRoIHRoZSBwcmV2aW91cyBub2RlLFxuICAgIC8vIGp1c3QgYXNzaWduIHRoZSB1cGRhdGVkIHNoYXJlZCBkaXNwYXRjaCBhbmQgd2VcdTIwMTlyZSBkb25lIVxuICAgIC8vIE90aGVyd2lzZSwgY29weS1vbi13cml0ZS5cbiAgICBpZiAob24gIT09IG9uMCkgKG9uMSA9IChvbjAgPSBvbikuY29weSgpKS5vbihuYW1lLCBsaXN0ZW5lcik7XG5cbiAgICBzY2hlZHVsZS5vbiA9IG9uMTtcbiAgfTtcbn1cblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24obmFtZSwgbGlzdGVuZXIpIHtcbiAgdmFyIGlkID0gdGhpcy5faWQ7XG5cbiAgcmV0dXJuIGFyZ3VtZW50cy5sZW5ndGggPCAyXG4gICAgICA/IGdldCh0aGlzLm5vZGUoKSwgaWQpLm9uLm9uKG5hbWUpXG4gICAgICA6IHRoaXMuZWFjaChvbkZ1bmN0aW9uKGlkLCBuYW1lLCBsaXN0ZW5lcikpO1xufVxuIiwgImZ1bmN0aW9uIHJlbW92ZUZ1bmN0aW9uKGlkKSB7XG4gIHJldHVybiBmdW5jdGlvbigpIHtcbiAgICB2YXIgcGFyZW50ID0gdGhpcy5wYXJlbnROb2RlO1xuICAgIGZvciAodmFyIGkgaW4gdGhpcy5fX3RyYW5zaXRpb24pIGlmICgraSAhPT0gaWQpIHJldHVybjtcbiAgICBpZiAocGFyZW50KSBwYXJlbnQucmVtb3ZlQ2hpbGQodGhpcyk7XG4gIH07XG59XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKCkge1xuICByZXR1cm4gdGhpcy5vbihcImVuZC5yZW1vdmVcIiwgcmVtb3ZlRnVuY3Rpb24odGhpcy5faWQpKTtcbn1cbiIsICJpbXBvcnQge3NlbGVjdG9yfSBmcm9tIFwiZDMtc2VsZWN0aW9uXCI7XG5pbXBvcnQge1RyYW5zaXRpb259IGZyb20gXCIuL2luZGV4LmpzXCI7XG5pbXBvcnQgc2NoZWR1bGUsIHtnZXR9IGZyb20gXCIuL3NjaGVkdWxlLmpzXCI7XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKHNlbGVjdCkge1xuICB2YXIgbmFtZSA9IHRoaXMuX25hbWUsXG4gICAgICBpZCA9IHRoaXMuX2lkO1xuXG4gIGlmICh0eXBlb2Ygc2VsZWN0ICE9PSBcImZ1bmN0aW9uXCIpIHNlbGVjdCA9IHNlbGVjdG9yKHNlbGVjdCk7XG5cbiAgZm9yICh2YXIgZ3JvdXBzID0gdGhpcy5fZ3JvdXBzLCBtID0gZ3JvdXBzLmxlbmd0aCwgc3ViZ3JvdXBzID0gbmV3IEFycmF5KG0pLCBqID0gMDsgaiA8IG07ICsraikge1xuICAgIGZvciAodmFyIGdyb3VwID0gZ3JvdXBzW2pdLCBuID0gZ3JvdXAubGVuZ3RoLCBzdWJncm91cCA9IHN1Ymdyb3Vwc1tqXSA9IG5ldyBBcnJheShuKSwgbm9kZSwgc3Vibm9kZSwgaSA9IDA7IGkgPCBuOyArK2kpIHtcbiAgICAgIGlmICgobm9kZSA9IGdyb3VwW2ldKSAmJiAoc3Vibm9kZSA9IHNlbGVjdC5jYWxsKG5vZGUsIG5vZGUuX19kYXRhX18sIGksIGdyb3VwKSkpIHtcbiAgICAgICAgaWYgKFwiX19kYXRhX19cIiBpbiBub2RlKSBzdWJub2RlLl9fZGF0YV9fID0gbm9kZS5fX2RhdGFfXztcbiAgICAgICAgc3ViZ3JvdXBbaV0gPSBzdWJub2RlO1xuICAgICAgICBzY2hlZHVsZShzdWJncm91cFtpXSwgbmFtZSwgaWQsIGksIHN1Ymdyb3VwLCBnZXQobm9kZSwgaWQpKTtcbiAgICAgIH1cbiAgICB9XG4gIH1cblxuICByZXR1cm4gbmV3IFRyYW5zaXRpb24oc3ViZ3JvdXBzLCB0aGlzLl9wYXJlbnRzLCBuYW1lLCBpZCk7XG59XG4iLCAiaW1wb3J0IHtzZWxlY3RvckFsbH0gZnJvbSBcImQzLXNlbGVjdGlvblwiO1xuaW1wb3J0IHtUcmFuc2l0aW9ufSBmcm9tIFwiLi9pbmRleC5qc1wiO1xuaW1wb3J0IHNjaGVkdWxlLCB7Z2V0fSBmcm9tIFwiLi9zY2hlZHVsZS5qc1wiO1xuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbihzZWxlY3QpIHtcbiAgdmFyIG5hbWUgPSB0aGlzLl9uYW1lLFxuICAgICAgaWQgPSB0aGlzLl9pZDtcblxuICBpZiAodHlwZW9mIHNlbGVjdCAhPT0gXCJmdW5jdGlvblwiKSBzZWxlY3QgPSBzZWxlY3RvckFsbChzZWxlY3QpO1xuXG4gIGZvciAodmFyIGdyb3VwcyA9IHRoaXMuX2dyb3VwcywgbSA9IGdyb3Vwcy5sZW5ndGgsIHN1Ymdyb3VwcyA9IFtdLCBwYXJlbnRzID0gW10sIGogPSAwOyBqIDwgbTsgKytqKSB7XG4gICAgZm9yICh2YXIgZ3JvdXAgPSBncm91cHNbal0sIG4gPSBncm91cC5sZW5ndGgsIG5vZGUsIGkgPSAwOyBpIDwgbjsgKytpKSB7XG4gICAgICBpZiAobm9kZSA9IGdyb3VwW2ldKSB7XG4gICAgICAgIGZvciAodmFyIGNoaWxkcmVuID0gc2VsZWN0LmNhbGwobm9kZSwgbm9kZS5fX2RhdGFfXywgaSwgZ3JvdXApLCBjaGlsZCwgaW5oZXJpdCA9IGdldChub2RlLCBpZCksIGsgPSAwLCBsID0gY2hpbGRyZW4ubGVuZ3RoOyBrIDwgbDsgKytrKSB7XG4gICAgICAgICAgaWYgKGNoaWxkID0gY2hpbGRyZW5ba10pIHtcbiAgICAgICAgICAgIHNjaGVkdWxlKGNoaWxkLCBuYW1lLCBpZCwgaywgY2hpbGRyZW4sIGluaGVyaXQpO1xuICAgICAgICAgIH1cbiAgICAgICAgfVxuICAgICAgICBzdWJncm91cHMucHVzaChjaGlsZHJlbik7XG4gICAgICAgIHBhcmVudHMucHVzaChub2RlKTtcbiAgICAgIH1cbiAgICB9XG4gIH1cblxuICByZXR1cm4gbmV3IFRyYW5zaXRpb24oc3ViZ3JvdXBzLCBwYXJlbnRzLCBuYW1lLCBpZCk7XG59XG4iLCAiaW1wb3J0IHtzZWxlY3Rpb259IGZyb20gXCJkMy1zZWxlY3Rpb25cIjtcblxudmFyIFNlbGVjdGlvbiA9IHNlbGVjdGlvbi5wcm90b3R5cGUuY29uc3RydWN0b3I7XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKCkge1xuICByZXR1cm4gbmV3IFNlbGVjdGlvbih0aGlzLl9ncm91cHMsIHRoaXMuX3BhcmVudHMpO1xufVxuIiwgImltcG9ydCB7aW50ZXJwb2xhdGVUcmFuc2Zvcm1Dc3MgYXMgaW50ZXJwb2xhdGVUcmFuc2Zvcm19IGZyb20gXCJkMy1pbnRlcnBvbGF0ZVwiO1xuaW1wb3J0IHtzdHlsZX0gZnJvbSBcImQzLXNlbGVjdGlvblwiO1xuaW1wb3J0IHtzZXR9IGZyb20gXCIuL3NjaGVkdWxlLmpzXCI7XG5pbXBvcnQge3R3ZWVuVmFsdWV9IGZyb20gXCIuL3R3ZWVuLmpzXCI7XG5pbXBvcnQgaW50ZXJwb2xhdGUgZnJvbSBcIi4vaW50ZXJwb2xhdGUuanNcIjtcblxuZnVuY3Rpb24gc3R5bGVOdWxsKG5hbWUsIGludGVycG9sYXRlKSB7XG4gIHZhciBzdHJpbmcwMCxcbiAgICAgIHN0cmluZzEwLFxuICAgICAgaW50ZXJwb2xhdGUwO1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgdmFyIHN0cmluZzAgPSBzdHlsZSh0aGlzLCBuYW1lKSxcbiAgICAgICAgc3RyaW5nMSA9ICh0aGlzLnN0eWxlLnJlbW92ZVByb3BlcnR5KG5hbWUpLCBzdHlsZSh0aGlzLCBuYW1lKSk7XG4gICAgcmV0dXJuIHN0cmluZzAgPT09IHN0cmluZzEgPyBudWxsXG4gICAgICAgIDogc3RyaW5nMCA9PT0gc3RyaW5nMDAgJiYgc3RyaW5nMSA9PT0gc3RyaW5nMTAgPyBpbnRlcnBvbGF0ZTBcbiAgICAgICAgOiBpbnRlcnBvbGF0ZTAgPSBpbnRlcnBvbGF0ZShzdHJpbmcwMCA9IHN0cmluZzAsIHN0cmluZzEwID0gc3RyaW5nMSk7XG4gIH07XG59XG5cbmZ1bmN0aW9uIHN0eWxlUmVtb3ZlKG5hbWUpIHtcbiAgcmV0dXJuIGZ1bmN0aW9uKCkge1xuICAgIHRoaXMuc3R5bGUucmVtb3ZlUHJvcGVydHkobmFtZSk7XG4gIH07XG59XG5cbmZ1bmN0aW9uIHN0eWxlQ29uc3RhbnQobmFtZSwgaW50ZXJwb2xhdGUsIHZhbHVlMSkge1xuICB2YXIgc3RyaW5nMDAsXG4gICAgICBzdHJpbmcxID0gdmFsdWUxICsgXCJcIixcbiAgICAgIGludGVycG9sYXRlMDtcbiAgcmV0dXJuIGZ1bmN0aW9uKCkge1xuICAgIHZhciBzdHJpbmcwID0gc3R5bGUodGhpcywgbmFtZSk7XG4gICAgcmV0dXJuIHN0cmluZzAgPT09IHN0cmluZzEgPyBudWxsXG4gICAgICAgIDogc3RyaW5nMCA9PT0gc3RyaW5nMDAgPyBpbnRlcnBvbGF0ZTBcbiAgICAgICAgOiBpbnRlcnBvbGF0ZTAgPSBpbnRlcnBvbGF0ZShzdHJpbmcwMCA9IHN0cmluZzAsIHZhbHVlMSk7XG4gIH07XG59XG5cbmZ1bmN0aW9uIHN0eWxlRnVuY3Rpb24obmFtZSwgaW50ZXJwb2xhdGUsIHZhbHVlKSB7XG4gIHZhciBzdHJpbmcwMCxcbiAgICAgIHN0cmluZzEwLFxuICAgICAgaW50ZXJwb2xhdGUwO1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgdmFyIHN0cmluZzAgPSBzdHlsZSh0aGlzLCBuYW1lKSxcbiAgICAgICAgdmFsdWUxID0gdmFsdWUodGhpcyksXG4gICAgICAgIHN0cmluZzEgPSB2YWx1ZTEgKyBcIlwiO1xuICAgIGlmICh2YWx1ZTEgPT0gbnVsbCkgc3RyaW5nMSA9IHZhbHVlMSA9ICh0aGlzLnN0eWxlLnJlbW92ZVByb3BlcnR5KG5hbWUpLCBzdHlsZSh0aGlzLCBuYW1lKSk7XG4gICAgcmV0dXJuIHN0cmluZzAgPT09IHN0cmluZzEgPyBudWxsXG4gICAgICAgIDogc3RyaW5nMCA9PT0gc3RyaW5nMDAgJiYgc3RyaW5nMSA9PT0gc3RyaW5nMTAgPyBpbnRlcnBvbGF0ZTBcbiAgICAgICAgOiAoc3RyaW5nMTAgPSBzdHJpbmcxLCBpbnRlcnBvbGF0ZTAgPSBpbnRlcnBvbGF0ZShzdHJpbmcwMCA9IHN0cmluZzAsIHZhbHVlMSkpO1xuICB9O1xufVxuXG5mdW5jdGlvbiBzdHlsZU1heWJlUmVtb3ZlKGlkLCBuYW1lKSB7XG4gIHZhciBvbjAsIG9uMSwgbGlzdGVuZXIwLCBrZXkgPSBcInN0eWxlLlwiICsgbmFtZSwgZXZlbnQgPSBcImVuZC5cIiArIGtleSwgcmVtb3ZlO1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgdmFyIHNjaGVkdWxlID0gc2V0KHRoaXMsIGlkKSxcbiAgICAgICAgb24gPSBzY2hlZHVsZS5vbixcbiAgICAgICAgbGlzdGVuZXIgPSBzY2hlZHVsZS52YWx1ZVtrZXldID09IG51bGwgPyByZW1vdmUgfHwgKHJlbW92ZSA9IHN0eWxlUmVtb3ZlKG5hbWUpKSA6IHVuZGVmaW5lZDtcblxuICAgIC8vIElmIHRoaXMgbm9kZSBzaGFyZWQgYSBkaXNwYXRjaCB3aXRoIHRoZSBwcmV2aW91cyBub2RlLFxuICAgIC8vIGp1c3QgYXNzaWduIHRoZSB1cGRhdGVkIHNoYXJlZCBkaXNwYXRjaCBhbmQgd2VcdTIwMTlyZSBkb25lIVxuICAgIC8vIE90aGVyd2lzZSwgY29weS1vbi13cml0ZS5cbiAgICBpZiAob24gIT09IG9uMCB8fCBsaXN0ZW5lcjAgIT09IGxpc3RlbmVyKSAob24xID0gKG9uMCA9IG9uKS5jb3B5KCkpLm9uKGV2ZW50LCBsaXN0ZW5lcjAgPSBsaXN0ZW5lcik7XG5cbiAgICBzY2hlZHVsZS5vbiA9IG9uMTtcbiAgfTtcbn1cblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24obmFtZSwgdmFsdWUsIHByaW9yaXR5KSB7XG4gIHZhciBpID0gKG5hbWUgKz0gXCJcIikgPT09IFwidHJhbnNmb3JtXCIgPyBpbnRlcnBvbGF0ZVRyYW5zZm9ybSA6IGludGVycG9sYXRlO1xuICByZXR1cm4gdmFsdWUgPT0gbnVsbCA/IHRoaXNcbiAgICAgIC5zdHlsZVR3ZWVuKG5hbWUsIHN0eWxlTnVsbChuYW1lLCBpKSlcbiAgICAgIC5vbihcImVuZC5zdHlsZS5cIiArIG5hbWUsIHN0eWxlUmVtb3ZlKG5hbWUpKVxuICAgIDogdHlwZW9mIHZhbHVlID09PSBcImZ1bmN0aW9uXCIgPyB0aGlzXG4gICAgICAuc3R5bGVUd2VlbihuYW1lLCBzdHlsZUZ1bmN0aW9uKG5hbWUsIGksIHR3ZWVuVmFsdWUodGhpcywgXCJzdHlsZS5cIiArIG5hbWUsIHZhbHVlKSkpXG4gICAgICAuZWFjaChzdHlsZU1heWJlUmVtb3ZlKHRoaXMuX2lkLCBuYW1lKSlcbiAgICA6IHRoaXNcbiAgICAgIC5zdHlsZVR3ZWVuKG5hbWUsIHN0eWxlQ29uc3RhbnQobmFtZSwgaSwgdmFsdWUpLCBwcmlvcml0eSlcbiAgICAgIC5vbihcImVuZC5zdHlsZS5cIiArIG5hbWUsIG51bGwpO1xufVxuIiwgImZ1bmN0aW9uIHN0eWxlSW50ZXJwb2xhdGUobmFtZSwgaSwgcHJpb3JpdHkpIHtcbiAgcmV0dXJuIGZ1bmN0aW9uKHQpIHtcbiAgICB0aGlzLnN0eWxlLnNldFByb3BlcnR5KG5hbWUsIGkuY2FsbCh0aGlzLCB0KSwgcHJpb3JpdHkpO1xuICB9O1xufVxuXG5mdW5jdGlvbiBzdHlsZVR3ZWVuKG5hbWUsIHZhbHVlLCBwcmlvcml0eSkge1xuICB2YXIgdCwgaTA7XG4gIGZ1bmN0aW9uIHR3ZWVuKCkge1xuICAgIHZhciBpID0gdmFsdWUuYXBwbHkodGhpcywgYXJndW1lbnRzKTtcbiAgICBpZiAoaSAhPT0gaTApIHQgPSAoaTAgPSBpKSAmJiBzdHlsZUludGVycG9sYXRlKG5hbWUsIGksIHByaW9yaXR5KTtcbiAgICByZXR1cm4gdDtcbiAgfVxuICB0d2Vlbi5fdmFsdWUgPSB2YWx1ZTtcbiAgcmV0dXJuIHR3ZWVuO1xufVxuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbihuYW1lLCB2YWx1ZSwgcHJpb3JpdHkpIHtcbiAgdmFyIGtleSA9IFwic3R5bGUuXCIgKyAobmFtZSArPSBcIlwiKTtcbiAgaWYgKGFyZ3VtZW50cy5sZW5ndGggPCAyKSByZXR1cm4gKGtleSA9IHRoaXMudHdlZW4oa2V5KSkgJiYga2V5Ll92YWx1ZTtcbiAgaWYgKHZhbHVlID09IG51bGwpIHJldHVybiB0aGlzLnR3ZWVuKGtleSwgbnVsbCk7XG4gIGlmICh0eXBlb2YgdmFsdWUgIT09IFwiZnVuY3Rpb25cIikgdGhyb3cgbmV3IEVycm9yO1xuICByZXR1cm4gdGhpcy50d2VlbihrZXksIHN0eWxlVHdlZW4obmFtZSwgdmFsdWUsIHByaW9yaXR5ID09IG51bGwgPyBcIlwiIDogcHJpb3JpdHkpKTtcbn1cbiIsICJpbXBvcnQge3R3ZWVuVmFsdWV9IGZyb20gXCIuL3R3ZWVuLmpzXCI7XG5cbmZ1bmN0aW9uIHRleHRDb25zdGFudCh2YWx1ZSkge1xuICByZXR1cm4gZnVuY3Rpb24oKSB7XG4gICAgdGhpcy50ZXh0Q29udGVudCA9IHZhbHVlO1xuICB9O1xufVxuXG5mdW5jdGlvbiB0ZXh0RnVuY3Rpb24odmFsdWUpIHtcbiAgcmV0dXJuIGZ1bmN0aW9uKCkge1xuICAgIHZhciB2YWx1ZTEgPSB2YWx1ZSh0aGlzKTtcbiAgICB0aGlzLnRleHRDb250ZW50ID0gdmFsdWUxID09IG51bGwgPyBcIlwiIDogdmFsdWUxO1xuICB9O1xufVxuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbih2YWx1ZSkge1xuICByZXR1cm4gdGhpcy50d2VlbihcInRleHRcIiwgdHlwZW9mIHZhbHVlID09PSBcImZ1bmN0aW9uXCJcbiAgICAgID8gdGV4dEZ1bmN0aW9uKHR3ZWVuVmFsdWUodGhpcywgXCJ0ZXh0XCIsIHZhbHVlKSlcbiAgICAgIDogdGV4dENvbnN0YW50KHZhbHVlID09IG51bGwgPyBcIlwiIDogdmFsdWUgKyBcIlwiKSk7XG59XG4iLCAiZnVuY3Rpb24gdGV4dEludGVycG9sYXRlKGkpIHtcbiAgcmV0dXJuIGZ1bmN0aW9uKHQpIHtcbiAgICB0aGlzLnRleHRDb250ZW50ID0gaS5jYWxsKHRoaXMsIHQpO1xuICB9O1xufVxuXG5mdW5jdGlvbiB0ZXh0VHdlZW4odmFsdWUpIHtcbiAgdmFyIHQwLCBpMDtcbiAgZnVuY3Rpb24gdHdlZW4oKSB7XG4gICAgdmFyIGkgPSB2YWx1ZS5hcHBseSh0aGlzLCBhcmd1bWVudHMpO1xuICAgIGlmIChpICE9PSBpMCkgdDAgPSAoaTAgPSBpKSAmJiB0ZXh0SW50ZXJwb2xhdGUoaSk7XG4gICAgcmV0dXJuIHQwO1xuICB9XG4gIHR3ZWVuLl92YWx1ZSA9IHZhbHVlO1xuICByZXR1cm4gdHdlZW47XG59XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKHZhbHVlKSB7XG4gIHZhciBrZXkgPSBcInRleHRcIjtcbiAgaWYgKGFyZ3VtZW50cy5sZW5ndGggPCAxKSByZXR1cm4gKGtleSA9IHRoaXMudHdlZW4oa2V5KSkgJiYga2V5Ll92YWx1ZTtcbiAgaWYgKHZhbHVlID09IG51bGwpIHJldHVybiB0aGlzLnR3ZWVuKGtleSwgbnVsbCk7XG4gIGlmICh0eXBlb2YgdmFsdWUgIT09IFwiZnVuY3Rpb25cIikgdGhyb3cgbmV3IEVycm9yO1xuICByZXR1cm4gdGhpcy50d2VlbihrZXksIHRleHRUd2Vlbih2YWx1ZSkpO1xufVxuIiwgImltcG9ydCB7VHJhbnNpdGlvbiwgbmV3SWR9IGZyb20gXCIuL2luZGV4LmpzXCI7XG5pbXBvcnQgc2NoZWR1bGUsIHtnZXR9IGZyb20gXCIuL3NjaGVkdWxlLmpzXCI7XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKCkge1xuICB2YXIgbmFtZSA9IHRoaXMuX25hbWUsXG4gICAgICBpZDAgPSB0aGlzLl9pZCxcbiAgICAgIGlkMSA9IG5ld0lkKCk7XG5cbiAgZm9yICh2YXIgZ3JvdXBzID0gdGhpcy5fZ3JvdXBzLCBtID0gZ3JvdXBzLmxlbmd0aCwgaiA9IDA7IGogPCBtOyArK2opIHtcbiAgICBmb3IgKHZhciBncm91cCA9IGdyb3Vwc1tqXSwgbiA9IGdyb3VwLmxlbmd0aCwgbm9kZSwgaSA9IDA7IGkgPCBuOyArK2kpIHtcbiAgICAgIGlmIChub2RlID0gZ3JvdXBbaV0pIHtcbiAgICAgICAgdmFyIGluaGVyaXQgPSBnZXQobm9kZSwgaWQwKTtcbiAgICAgICAgc2NoZWR1bGUobm9kZSwgbmFtZSwgaWQxLCBpLCBncm91cCwge1xuICAgICAgICAgIHRpbWU6IGluaGVyaXQudGltZSArIGluaGVyaXQuZGVsYXkgKyBpbmhlcml0LmR1cmF0aW9uLFxuICAgICAgICAgIGRlbGF5OiAwLFxuICAgICAgICAgIGR1cmF0aW9uOiBpbmhlcml0LmR1cmF0aW9uLFxuICAgICAgICAgIGVhc2U6IGluaGVyaXQuZWFzZVxuICAgICAgICB9KTtcbiAgICAgIH1cbiAgICB9XG4gIH1cblxuICByZXR1cm4gbmV3IFRyYW5zaXRpb24oZ3JvdXBzLCB0aGlzLl9wYXJlbnRzLCBuYW1lLCBpZDEpO1xufVxuIiwgImltcG9ydCB7c2V0fSBmcm9tIFwiLi9zY2hlZHVsZS5qc1wiO1xuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbigpIHtcbiAgdmFyIG9uMCwgb24xLCB0aGF0ID0gdGhpcywgaWQgPSB0aGF0Ll9pZCwgc2l6ZSA9IHRoYXQuc2l6ZSgpO1xuICByZXR1cm4gbmV3IFByb21pc2UoZnVuY3Rpb24ocmVzb2x2ZSwgcmVqZWN0KSB7XG4gICAgdmFyIGNhbmNlbCA9IHt2YWx1ZTogcmVqZWN0fSxcbiAgICAgICAgZW5kID0ge3ZhbHVlOiBmdW5jdGlvbigpIHsgaWYgKC0tc2l6ZSA9PT0gMCkgcmVzb2x2ZSgpOyB9fTtcblxuICAgIHRoYXQuZWFjaChmdW5jdGlvbigpIHtcbiAgICAgIHZhciBzY2hlZHVsZSA9IHNldCh0aGlzLCBpZCksXG4gICAgICAgICAgb24gPSBzY2hlZHVsZS5vbjtcblxuICAgICAgLy8gSWYgdGhpcyBub2RlIHNoYXJlZCBhIGRpc3BhdGNoIHdpdGggdGhlIHByZXZpb3VzIG5vZGUsXG4gICAgICAvLyBqdXN0IGFzc2lnbiB0aGUgdXBkYXRlZCBzaGFyZWQgZGlzcGF0Y2ggYW5kIHdlXHUyMDE5cmUgZG9uZSFcbiAgICAgIC8vIE90aGVyd2lzZSwgY29weS1vbi13cml0ZS5cbiAgICAgIGlmIChvbiAhPT0gb24wKSB7XG4gICAgICAgIG9uMSA9IChvbjAgPSBvbikuY29weSgpO1xuICAgICAgICBvbjEuXy5jYW5jZWwucHVzaChjYW5jZWwpO1xuICAgICAgICBvbjEuXy5pbnRlcnJ1cHQucHVzaChjYW5jZWwpO1xuICAgICAgICBvbjEuXy5lbmQucHVzaChlbmQpO1xuICAgICAgfVxuXG4gICAgICBzY2hlZHVsZS5vbiA9IG9uMTtcbiAgICB9KTtcblxuICAgIC8vIFRoZSBzZWxlY3Rpb24gd2FzIGVtcHR5LCByZXNvbHZlIGVuZCBpbW1lZGlhdGVseVxuICAgIGlmIChzaXplID09PSAwKSByZXNvbHZlKCk7XG4gIH0pO1xufVxuIiwgImltcG9ydCB7c2VsZWN0aW9ufSBmcm9tIFwiZDMtc2VsZWN0aW9uXCI7XG5pbXBvcnQgdHJhbnNpdGlvbl9hdHRyIGZyb20gXCIuL2F0dHIuanNcIjtcbmltcG9ydCB0cmFuc2l0aW9uX2F0dHJUd2VlbiBmcm9tIFwiLi9hdHRyVHdlZW4uanNcIjtcbmltcG9ydCB0cmFuc2l0aW9uX2RlbGF5IGZyb20gXCIuL2RlbGF5LmpzXCI7XG5pbXBvcnQgdHJhbnNpdGlvbl9kdXJhdGlvbiBmcm9tIFwiLi9kdXJhdGlvbi5qc1wiO1xuaW1wb3J0IHRyYW5zaXRpb25fZWFzZSBmcm9tIFwiLi9lYXNlLmpzXCI7XG5pbXBvcnQgdHJhbnNpdGlvbl9lYXNlVmFyeWluZyBmcm9tIFwiLi9lYXNlVmFyeWluZy5qc1wiO1xuaW1wb3J0IHRyYW5zaXRpb25fZmlsdGVyIGZyb20gXCIuL2ZpbHRlci5qc1wiO1xuaW1wb3J0IHRyYW5zaXRpb25fbWVyZ2UgZnJvbSBcIi4vbWVyZ2UuanNcIjtcbmltcG9ydCB0cmFuc2l0aW9uX29uIGZyb20gXCIuL29uLmpzXCI7XG5pbXBvcnQgdHJhbnNpdGlvbl9yZW1vdmUgZnJvbSBcIi4vcmVtb3ZlLmpzXCI7XG5pbXBvcnQgdHJhbnNpdGlvbl9zZWxlY3QgZnJvbSBcIi4vc2VsZWN0LmpzXCI7XG5pbXBvcnQgdHJhbnNpdGlvbl9zZWxlY3RBbGwgZnJvbSBcIi4vc2VsZWN0QWxsLmpzXCI7XG5pbXBvcnQgdHJhbnNpdGlvbl9zZWxlY3Rpb24gZnJvbSBcIi4vc2VsZWN0aW9uLmpzXCI7XG5pbXBvcnQgdHJhbnNpdGlvbl9zdHlsZSBmcm9tIFwiLi9zdHlsZS5qc1wiO1xuaW1wb3J0IHRyYW5zaXRpb25fc3R5bGVUd2VlbiBmcm9tIFwiLi9zdHlsZVR3ZWVuLmpzXCI7XG5pbXBvcnQgdHJhbnNpdGlvbl90ZXh0IGZyb20gXCIuL3RleHQuanNcIjtcbmltcG9ydCB0cmFuc2l0aW9uX3RleHRUd2VlbiBmcm9tIFwiLi90ZXh0VHdlZW4uanNcIjtcbmltcG9ydCB0cmFuc2l0aW9uX3RyYW5zaXRpb24gZnJvbSBcIi4vdHJhbnNpdGlvbi5qc1wiO1xuaW1wb3J0IHRyYW5zaXRpb25fdHdlZW4gZnJvbSBcIi4vdHdlZW4uanNcIjtcbmltcG9ydCB0cmFuc2l0aW9uX2VuZCBmcm9tIFwiLi9lbmQuanNcIjtcblxudmFyIGlkID0gMDtcblxuZXhwb3J0IGZ1bmN0aW9uIFRyYW5zaXRpb24oZ3JvdXBzLCBwYXJlbnRzLCBuYW1lLCBpZCkge1xuICB0aGlzLl9ncm91cHMgPSBncm91cHM7XG4gIHRoaXMuX3BhcmVudHMgPSBwYXJlbnRzO1xuICB0aGlzLl9uYW1lID0gbmFtZTtcbiAgdGhpcy5faWQgPSBpZDtcbn1cblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24gdHJhbnNpdGlvbihuYW1lKSB7XG4gIHJldHVybiBzZWxlY3Rpb24oKS50cmFuc2l0aW9uKG5hbWUpO1xufVxuXG5leHBvcnQgZnVuY3Rpb24gbmV3SWQoKSB7XG4gIHJldHVybiArK2lkO1xufVxuXG52YXIgc2VsZWN0aW9uX3Byb3RvdHlwZSA9IHNlbGVjdGlvbi5wcm90b3R5cGU7XG5cblRyYW5zaXRpb24ucHJvdG90eXBlID0gdHJhbnNpdGlvbi5wcm90b3R5cGUgPSB7XG4gIGNvbnN0cnVjdG9yOiBUcmFuc2l0aW9uLFxuICBzZWxlY3Q6IHRyYW5zaXRpb25fc2VsZWN0LFxuICBzZWxlY3RBbGw6IHRyYW5zaXRpb25fc2VsZWN0QWxsLFxuICBzZWxlY3RDaGlsZDogc2VsZWN0aW9uX3Byb3RvdHlwZS5zZWxlY3RDaGlsZCxcbiAgc2VsZWN0Q2hpbGRyZW46IHNlbGVjdGlvbl9wcm90b3R5cGUuc2VsZWN0Q2hpbGRyZW4sXG4gIGZpbHRlcjogdHJhbnNpdGlvbl9maWx0ZXIsXG4gIG1lcmdlOiB0cmFuc2l0aW9uX21lcmdlLFxuICBzZWxlY3Rpb246IHRyYW5zaXRpb25fc2VsZWN0aW9uLFxuICB0cmFuc2l0aW9uOiB0cmFuc2l0aW9uX3RyYW5zaXRpb24sXG4gIGNhbGw6IHNlbGVjdGlvbl9wcm90b3R5cGUuY2FsbCxcbiAgbm9kZXM6IHNlbGVjdGlvbl9wcm90b3R5cGUubm9kZXMsXG4gIG5vZGU6IHNlbGVjdGlvbl9wcm90b3R5cGUubm9kZSxcbiAgc2l6ZTogc2VsZWN0aW9uX3Byb3RvdHlwZS5zaXplLFxuICBlbXB0eTogc2VsZWN0aW9uX3Byb3RvdHlwZS5lbXB0eSxcbiAgZWFjaDogc2VsZWN0aW9uX3Byb3RvdHlwZS5lYWNoLFxuICBvbjogdHJhbnNpdGlvbl9vbixcbiAgYXR0cjogdHJhbnNpdGlvbl9hdHRyLFxuICBhdHRyVHdlZW46IHRyYW5zaXRpb25fYXR0clR3ZWVuLFxuICBzdHlsZTogdHJhbnNpdGlvbl9zdHlsZSxcbiAgc3R5bGVUd2VlbjogdHJhbnNpdGlvbl9zdHlsZVR3ZWVuLFxuICB0ZXh0OiB0cmFuc2l0aW9uX3RleHQsXG4gIHRleHRUd2VlbjogdHJhbnNpdGlvbl90ZXh0VHdlZW4sXG4gIHJlbW92ZTogdHJhbnNpdGlvbl9yZW1vdmUsXG4gIHR3ZWVuOiB0cmFuc2l0aW9uX3R3ZWVuLFxuICBkZWxheTogdHJhbnNpdGlvbl9kZWxheSxcbiAgZHVyYXRpb246IHRyYW5zaXRpb25fZHVyYXRpb24sXG4gIGVhc2U6IHRyYW5zaXRpb25fZWFzZSxcbiAgZWFzZVZhcnlpbmc6IHRyYW5zaXRpb25fZWFzZVZhcnlpbmcsXG4gIGVuZDogdHJhbnNpdGlvbl9lbmQsXG4gIFtTeW1ib2wuaXRlcmF0b3JdOiBzZWxlY3Rpb25fcHJvdG90eXBlW1N5bWJvbC5pdGVyYXRvcl1cbn07XG4iLCAiZXhwb3J0IGZ1bmN0aW9uIGN1YmljSW4odCkge1xuICByZXR1cm4gdCAqIHQgKiB0O1xufVxuXG5leHBvcnQgZnVuY3Rpb24gY3ViaWNPdXQodCkge1xuICByZXR1cm4gLS10ICogdCAqIHQgKyAxO1xufVxuXG5leHBvcnQgZnVuY3Rpb24gY3ViaWNJbk91dCh0KSB7XG4gIHJldHVybiAoKHQgKj0gMikgPD0gMSA/IHQgKiB0ICogdCA6ICh0IC09IDIpICogdCAqIHQgKyAyKSAvIDI7XG59XG4iLCAiaW1wb3J0IHtUcmFuc2l0aW9uLCBuZXdJZH0gZnJvbSBcIi4uL3RyYW5zaXRpb24vaW5kZXguanNcIjtcbmltcG9ydCBzY2hlZHVsZSBmcm9tIFwiLi4vdHJhbnNpdGlvbi9zY2hlZHVsZS5qc1wiO1xuaW1wb3J0IHtlYXNlQ3ViaWNJbk91dH0gZnJvbSBcImQzLWVhc2VcIjtcbmltcG9ydCB7bm93fSBmcm9tIFwiZDMtdGltZXJcIjtcblxudmFyIGRlZmF1bHRUaW1pbmcgPSB7XG4gIHRpbWU6IG51bGwsIC8vIFNldCBvbiB1c2UuXG4gIGRlbGF5OiAwLFxuICBkdXJhdGlvbjogMjUwLFxuICBlYXNlOiBlYXNlQ3ViaWNJbk91dFxufTtcblxuZnVuY3Rpb24gaW5oZXJpdChub2RlLCBpZCkge1xuICB2YXIgdGltaW5nO1xuICB3aGlsZSAoISh0aW1pbmcgPSBub2RlLl9fdHJhbnNpdGlvbikgfHwgISh0aW1pbmcgPSB0aW1pbmdbaWRdKSkge1xuICAgIGlmICghKG5vZGUgPSBub2RlLnBhcmVudE5vZGUpKSB7XG4gICAgICB0aHJvdyBuZXcgRXJyb3IoYHRyYW5zaXRpb24gJHtpZH0gbm90IGZvdW5kYCk7XG4gICAgfVxuICB9XG4gIHJldHVybiB0aW1pbmc7XG59XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKG5hbWUpIHtcbiAgdmFyIGlkLFxuICAgICAgdGltaW5nO1xuXG4gIGlmIChuYW1lIGluc3RhbmNlb2YgVHJhbnNpdGlvbikge1xuICAgIGlkID0gbmFtZS5faWQsIG5hbWUgPSBuYW1lLl9uYW1lO1xuICB9IGVsc2Uge1xuICAgIGlkID0gbmV3SWQoKSwgKHRpbWluZyA9IGRlZmF1bHRUaW1pbmcpLnRpbWUgPSBub3coKSwgbmFtZSA9IG5hbWUgPT0gbnVsbCA/IG51bGwgOiBuYW1lICsgXCJcIjtcbiAgfVxuXG4gIGZvciAodmFyIGdyb3VwcyA9IHRoaXMuX2dyb3VwcywgbSA9IGdyb3Vwcy5sZW5ndGgsIGogPSAwOyBqIDwgbTsgKytqKSB7XG4gICAgZm9yICh2YXIgZ3JvdXAgPSBncm91cHNbal0sIG4gPSBncm91cC5sZW5ndGgsIG5vZGUsIGkgPSAwOyBpIDwgbjsgKytpKSB7XG4gICAgICBpZiAobm9kZSA9IGdyb3VwW2ldKSB7XG4gICAgICAgIHNjaGVkdWxlKG5vZGUsIG5hbWUsIGlkLCBpLCBncm91cCwgdGltaW5nIHx8IGluaGVyaXQobm9kZSwgaWQpKTtcbiAgICAgIH1cbiAgICB9XG4gIH1cblxuICByZXR1cm4gbmV3IFRyYW5zaXRpb24oZ3JvdXBzLCB0aGlzLl9wYXJlbnRzLCBuYW1lLCBpZCk7XG59XG4iLCAiaW1wb3J0IHtzZWxlY3Rpb259IGZyb20gXCJkMy1zZWxlY3Rpb25cIjtcbmltcG9ydCBzZWxlY3Rpb25faW50ZXJydXB0IGZyb20gXCIuL2ludGVycnVwdC5qc1wiO1xuaW1wb3J0IHNlbGVjdGlvbl90cmFuc2l0aW9uIGZyb20gXCIuL3RyYW5zaXRpb24uanNcIjtcblxuc2VsZWN0aW9uLnByb3RvdHlwZS5pbnRlcnJ1cHQgPSBzZWxlY3Rpb25faW50ZXJydXB0O1xuc2VsZWN0aW9uLnByb3RvdHlwZS50cmFuc2l0aW9uID0gc2VsZWN0aW9uX3RyYW5zaXRpb247XG4iLCAiaW1wb3J0IHtkaXNwYXRjaH0gZnJvbSBcImQzLWRpc3BhdGNoXCI7XG5pbXBvcnQge2RyYWdEaXNhYmxlLCBkcmFnRW5hYmxlfSBmcm9tIFwiZDMtZHJhZ1wiO1xuaW1wb3J0IHtpbnRlcnBvbGF0ZX0gZnJvbSBcImQzLWludGVycG9sYXRlXCI7XG5pbXBvcnQge3BvaW50ZXIsIHNlbGVjdH0gZnJvbSBcImQzLXNlbGVjdGlvblwiO1xuaW1wb3J0IHtpbnRlcnJ1cHR9IGZyb20gXCJkMy10cmFuc2l0aW9uXCI7XG5pbXBvcnQgY29uc3RhbnQgZnJvbSBcIi4vY29uc3RhbnQuanNcIjtcbmltcG9ydCBCcnVzaEV2ZW50IGZyb20gXCIuL2V2ZW50LmpzXCI7XG5pbXBvcnQgbm9ldmVudCwge25vcHJvcGFnYXRpb259IGZyb20gXCIuL25vZXZlbnQuanNcIjtcblxudmFyIE1PREVfRFJBRyA9IHtuYW1lOiBcImRyYWdcIn0sXG4gICAgTU9ERV9TUEFDRSA9IHtuYW1lOiBcInNwYWNlXCJ9LFxuICAgIE1PREVfSEFORExFID0ge25hbWU6IFwiaGFuZGxlXCJ9LFxuICAgIE1PREVfQ0VOVEVSID0ge25hbWU6IFwiY2VudGVyXCJ9O1xuXG5jb25zdCB7YWJzLCBtYXgsIG1pbn0gPSBNYXRoO1xuXG5mdW5jdGlvbiBudW1iZXIxKGUpIHtcbiAgcmV0dXJuIFsrZVswXSwgK2VbMV1dO1xufVxuXG5mdW5jdGlvbiBudW1iZXIyKGUpIHtcbiAgcmV0dXJuIFtudW1iZXIxKGVbMF0pLCBudW1iZXIxKGVbMV0pXTtcbn1cblxudmFyIFggPSB7XG4gIG5hbWU6IFwieFwiLFxuICBoYW5kbGVzOiBbXCJ3XCIsIFwiZVwiXS5tYXAodHlwZSksXG4gIGlucHV0OiBmdW5jdGlvbih4LCBlKSB7IHJldHVybiB4ID09IG51bGwgPyBudWxsIDogW1sreFswXSwgZVswXVsxXV0sIFsreFsxXSwgZVsxXVsxXV1dOyB9LFxuICBvdXRwdXQ6IGZ1bmN0aW9uKHh5KSB7IHJldHVybiB4eSAmJiBbeHlbMF1bMF0sIHh5WzFdWzBdXTsgfVxufTtcblxudmFyIFkgPSB7XG4gIG5hbWU6IFwieVwiLFxuICBoYW5kbGVzOiBbXCJuXCIsIFwic1wiXS5tYXAodHlwZSksXG4gIGlucHV0OiBmdW5jdGlvbih5LCBlKSB7IHJldHVybiB5ID09IG51bGwgPyBudWxsIDogW1tlWzBdWzBdLCAreVswXV0sIFtlWzFdWzBdLCAreVsxXV1dOyB9LFxuICBvdXRwdXQ6IGZ1bmN0aW9uKHh5KSB7IHJldHVybiB4eSAmJiBbeHlbMF1bMV0sIHh5WzFdWzFdXTsgfVxufTtcblxudmFyIFhZID0ge1xuICBuYW1lOiBcInh5XCIsXG4gIGhhbmRsZXM6IFtcIm5cIiwgXCJ3XCIsIFwiZVwiLCBcInNcIiwgXCJud1wiLCBcIm5lXCIsIFwic3dcIiwgXCJzZVwiXS5tYXAodHlwZSksXG4gIGlucHV0OiBmdW5jdGlvbih4eSkgeyByZXR1cm4geHkgPT0gbnVsbCA/IG51bGwgOiBudW1iZXIyKHh5KTsgfSxcbiAgb3V0cHV0OiBmdW5jdGlvbih4eSkgeyByZXR1cm4geHk7IH1cbn07XG5cbnZhciBjdXJzb3JzID0ge1xuICBvdmVybGF5OiBcImNyb3NzaGFpclwiLFxuICBzZWxlY3Rpb246IFwibW92ZVwiLFxuICBuOiBcIm5zLXJlc2l6ZVwiLFxuICBlOiBcImV3LXJlc2l6ZVwiLFxuICBzOiBcIm5zLXJlc2l6ZVwiLFxuICB3OiBcImV3LXJlc2l6ZVwiLFxuICBudzogXCJud3NlLXJlc2l6ZVwiLFxuICBuZTogXCJuZXN3LXJlc2l6ZVwiLFxuICBzZTogXCJud3NlLXJlc2l6ZVwiLFxuICBzdzogXCJuZXN3LXJlc2l6ZVwiXG59O1xuXG52YXIgZmxpcFggPSB7XG4gIGU6IFwid1wiLFxuICB3OiBcImVcIixcbiAgbnc6IFwibmVcIixcbiAgbmU6IFwibndcIixcbiAgc2U6IFwic3dcIixcbiAgc3c6IFwic2VcIlxufTtcblxudmFyIGZsaXBZID0ge1xuICBuOiBcInNcIixcbiAgczogXCJuXCIsXG4gIG53OiBcInN3XCIsXG4gIG5lOiBcInNlXCIsXG4gIHNlOiBcIm5lXCIsXG4gIHN3OiBcIm53XCJcbn07XG5cbnZhciBzaWduc1ggPSB7XG4gIG92ZXJsYXk6ICsxLFxuICBzZWxlY3Rpb246ICsxLFxuICBuOiBudWxsLFxuICBlOiArMSxcbiAgczogbnVsbCxcbiAgdzogLTEsXG4gIG53OiAtMSxcbiAgbmU6ICsxLFxuICBzZTogKzEsXG4gIHN3OiAtMVxufTtcblxudmFyIHNpZ25zWSA9IHtcbiAgb3ZlcmxheTogKzEsXG4gIHNlbGVjdGlvbjogKzEsXG4gIG46IC0xLFxuICBlOiBudWxsLFxuICBzOiArMSxcbiAgdzogbnVsbCxcbiAgbnc6IC0xLFxuICBuZTogLTEsXG4gIHNlOiArMSxcbiAgc3c6ICsxXG59O1xuXG5mdW5jdGlvbiB0eXBlKHQpIHtcbiAgcmV0dXJuIHt0eXBlOiB0fTtcbn1cblxuLy8gSWdub3JlIHJpZ2h0LWNsaWNrLCBzaW5jZSB0aGF0IHNob3VsZCBvcGVuIHRoZSBjb250ZXh0IG1lbnUuXG5mdW5jdGlvbiBkZWZhdWx0RmlsdGVyKGV2ZW50KSB7XG4gIHJldHVybiAhZXZlbnQuY3RybEtleSAmJiAhZXZlbnQuYnV0dG9uO1xufVxuXG5mdW5jdGlvbiBkZWZhdWx0RXh0ZW50KCkge1xuICB2YXIgc3ZnID0gdGhpcy5vd25lclNWR0VsZW1lbnQgfHwgdGhpcztcbiAgaWYgKHN2Zy5oYXNBdHRyaWJ1dGUoXCJ2aWV3Qm94XCIpKSB7XG4gICAgc3ZnID0gc3ZnLnZpZXdCb3guYmFzZVZhbDtcbiAgICByZXR1cm4gW1tzdmcueCwgc3ZnLnldLCBbc3ZnLnggKyBzdmcud2lkdGgsIHN2Zy55ICsgc3ZnLmhlaWdodF1dO1xuICB9XG4gIHJldHVybiBbWzAsIDBdLCBbc3ZnLndpZHRoLmJhc2VWYWwudmFsdWUsIHN2Zy5oZWlnaHQuYmFzZVZhbC52YWx1ZV1dO1xufVxuXG5mdW5jdGlvbiBkZWZhdWx0VG91Y2hhYmxlKCkge1xuICByZXR1cm4gbmF2aWdhdG9yLm1heFRvdWNoUG9pbnRzIHx8IChcIm9udG91Y2hzdGFydFwiIGluIHRoaXMpO1xufVxuXG4vLyBMaWtlIGQzLmxvY2FsLCBidXQgd2l0aCB0aGUgbmFtZSBcdTIwMUNfX2JydXNoXHUyMDFEIHJhdGhlciB0aGFuIGF1dG8tZ2VuZXJhdGVkLlxuZnVuY3Rpb24gbG9jYWwobm9kZSkge1xuICB3aGlsZSAoIW5vZGUuX19icnVzaCkgaWYgKCEobm9kZSA9IG5vZGUucGFyZW50Tm9kZSkpIHJldHVybjtcbiAgcmV0dXJuIG5vZGUuX19icnVzaDtcbn1cblxuZnVuY3Rpb24gZW1wdHkoZXh0ZW50KSB7XG4gIHJldHVybiBleHRlbnRbMF1bMF0gPT09IGV4dGVudFsxXVswXVxuICAgICAgfHwgZXh0ZW50WzBdWzFdID09PSBleHRlbnRbMV1bMV07XG59XG5cbmV4cG9ydCBmdW5jdGlvbiBicnVzaFNlbGVjdGlvbihub2RlKSB7XG4gIHZhciBzdGF0ZSA9IG5vZGUuX19icnVzaDtcbiAgcmV0dXJuIHN0YXRlID8gc3RhdGUuZGltLm91dHB1dChzdGF0ZS5zZWxlY3Rpb24pIDogbnVsbDtcbn1cblxuZXhwb3J0IGZ1bmN0aW9uIGJydXNoWCgpIHtcbiAgcmV0dXJuIGJydXNoKFgpO1xufVxuXG5leHBvcnQgZnVuY3Rpb24gYnJ1c2hZKCkge1xuICByZXR1cm4gYnJ1c2goWSk7XG59XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKCkge1xuICByZXR1cm4gYnJ1c2goWFkpO1xufVxuXG5mdW5jdGlvbiBicnVzaChkaW0pIHtcbiAgdmFyIGV4dGVudCA9IGRlZmF1bHRFeHRlbnQsXG4gICAgICBmaWx0ZXIgPSBkZWZhdWx0RmlsdGVyLFxuICAgICAgdG91Y2hhYmxlID0gZGVmYXVsdFRvdWNoYWJsZSxcbiAgICAgIGtleXMgPSB0cnVlLFxuICAgICAgbGlzdGVuZXJzID0gZGlzcGF0Y2goXCJzdGFydFwiLCBcImJydXNoXCIsIFwiZW5kXCIpLFxuICAgICAgaGFuZGxlU2l6ZSA9IDYsXG4gICAgICB0b3VjaGVuZGluZztcblxuICBmdW5jdGlvbiBicnVzaChncm91cCkge1xuICAgIHZhciBvdmVybGF5ID0gZ3JvdXBcbiAgICAgICAgLnByb3BlcnR5KFwiX19icnVzaFwiLCBpbml0aWFsaXplKVxuICAgICAgLnNlbGVjdEFsbChcIi5vdmVybGF5XCIpXG4gICAgICAuZGF0YShbdHlwZShcIm92ZXJsYXlcIildKTtcblxuICAgIG92ZXJsYXkuZW50ZXIoKS5hcHBlbmQoXCJyZWN0XCIpXG4gICAgICAgIC5hdHRyKFwiY2xhc3NcIiwgXCJvdmVybGF5XCIpXG4gICAgICAgIC5hdHRyKFwicG9pbnRlci1ldmVudHNcIiwgXCJhbGxcIilcbiAgICAgICAgLmF0dHIoXCJjdXJzb3JcIiwgY3Vyc29ycy5vdmVybGF5KVxuICAgICAgLm1lcmdlKG92ZXJsYXkpXG4gICAgICAgIC5lYWNoKGZ1bmN0aW9uKCkge1xuICAgICAgICAgIHZhciBleHRlbnQgPSBsb2NhbCh0aGlzKS5leHRlbnQ7XG4gICAgICAgICAgc2VsZWN0KHRoaXMpXG4gICAgICAgICAgICAgIC5hdHRyKFwieFwiLCBleHRlbnRbMF1bMF0pXG4gICAgICAgICAgICAgIC5hdHRyKFwieVwiLCBleHRlbnRbMF1bMV0pXG4gICAgICAgICAgICAgIC5hdHRyKFwid2lkdGhcIiwgZXh0ZW50WzFdWzBdIC0gZXh0ZW50WzBdWzBdKVxuICAgICAgICAgICAgICAuYXR0cihcImhlaWdodFwiLCBleHRlbnRbMV1bMV0gLSBleHRlbnRbMF1bMV0pO1xuICAgICAgICB9KTtcblxuICAgIGdyb3VwLnNlbGVjdEFsbChcIi5zZWxlY3Rpb25cIilcbiAgICAgIC5kYXRhKFt0eXBlKFwic2VsZWN0aW9uXCIpXSlcbiAgICAgIC5lbnRlcigpLmFwcGVuZChcInJlY3RcIilcbiAgICAgICAgLmF0dHIoXCJjbGFzc1wiLCBcInNlbGVjdGlvblwiKVxuICAgICAgICAuYXR0cihcImN1cnNvclwiLCBjdXJzb3JzLnNlbGVjdGlvbilcbiAgICAgICAgLmF0dHIoXCJmaWxsXCIsIFwiIzc3N1wiKVxuICAgICAgICAuYXR0cihcImZpbGwtb3BhY2l0eVwiLCAwLjMpXG4gICAgICAgIC5hdHRyKFwic3Ryb2tlXCIsIFwiI2ZmZlwiKVxuICAgICAgICAuYXR0cihcInNoYXBlLXJlbmRlcmluZ1wiLCBcImNyaXNwRWRnZXNcIik7XG5cbiAgICB2YXIgaGFuZGxlID0gZ3JvdXAuc2VsZWN0QWxsKFwiLmhhbmRsZVwiKVxuICAgICAgLmRhdGEoZGltLmhhbmRsZXMsIGZ1bmN0aW9uKGQpIHsgcmV0dXJuIGQudHlwZTsgfSk7XG5cbiAgICBoYW5kbGUuZXhpdCgpLnJlbW92ZSgpO1xuXG4gICAgaGFuZGxlLmVudGVyKCkuYXBwZW5kKFwicmVjdFwiKVxuICAgICAgICAuYXR0cihcImNsYXNzXCIsIGZ1bmN0aW9uKGQpIHsgcmV0dXJuIFwiaGFuZGxlIGhhbmRsZS0tXCIgKyBkLnR5cGU7IH0pXG4gICAgICAgIC5hdHRyKFwiY3Vyc29yXCIsIGZ1bmN0aW9uKGQpIHsgcmV0dXJuIGN1cnNvcnNbZC50eXBlXTsgfSk7XG5cbiAgICBncm91cFxuICAgICAgICAuZWFjaChyZWRyYXcpXG4gICAgICAgIC5hdHRyKFwiZmlsbFwiLCBcIm5vbmVcIilcbiAgICAgICAgLmF0dHIoXCJwb2ludGVyLWV2ZW50c1wiLCBcImFsbFwiKVxuICAgICAgICAub24oXCJtb3VzZWRvd24uYnJ1c2hcIiwgc3RhcnRlZClcbiAgICAgIC5maWx0ZXIodG91Y2hhYmxlKVxuICAgICAgICAub24oXCJ0b3VjaHN0YXJ0LmJydXNoXCIsIHN0YXJ0ZWQpXG4gICAgICAgIC5vbihcInRvdWNobW92ZS5icnVzaFwiLCB0b3VjaG1vdmVkKVxuICAgICAgICAub24oXCJ0b3VjaGVuZC5icnVzaCB0b3VjaGNhbmNlbC5icnVzaFwiLCB0b3VjaGVuZGVkKVxuICAgICAgICAuc3R5bGUoXCJ0b3VjaC1hY3Rpb25cIiwgXCJub25lXCIpXG4gICAgICAgIC5zdHlsZShcIi13ZWJraXQtdGFwLWhpZ2hsaWdodC1jb2xvclwiLCBcInJnYmEoMCwwLDAsMClcIik7XG4gIH1cblxuICBicnVzaC5tb3ZlID0gZnVuY3Rpb24oZ3JvdXAsIHNlbGVjdGlvbiwgZXZlbnQpIHtcbiAgICBpZiAoZ3JvdXAudHdlZW4pIHtcbiAgICAgIGdyb3VwXG4gICAgICAgICAgLm9uKFwic3RhcnQuYnJ1c2hcIiwgZnVuY3Rpb24oZXZlbnQpIHsgZW1pdHRlcih0aGlzLCBhcmd1bWVudHMpLmJlZm9yZXN0YXJ0KCkuc3RhcnQoZXZlbnQpOyB9KVxuICAgICAgICAgIC5vbihcImludGVycnVwdC5icnVzaCBlbmQuYnJ1c2hcIiwgZnVuY3Rpb24oZXZlbnQpIHsgZW1pdHRlcih0aGlzLCBhcmd1bWVudHMpLmVuZChldmVudCk7IH0pXG4gICAgICAgICAgLnR3ZWVuKFwiYnJ1c2hcIiwgZnVuY3Rpb24oKSB7XG4gICAgICAgICAgICB2YXIgdGhhdCA9IHRoaXMsXG4gICAgICAgICAgICAgICAgc3RhdGUgPSB0aGF0Ll9fYnJ1c2gsXG4gICAgICAgICAgICAgICAgZW1pdCA9IGVtaXR0ZXIodGhhdCwgYXJndW1lbnRzKSxcbiAgICAgICAgICAgICAgICBzZWxlY3Rpb24wID0gc3RhdGUuc2VsZWN0aW9uLFxuICAgICAgICAgICAgICAgIHNlbGVjdGlvbjEgPSBkaW0uaW5wdXQodHlwZW9mIHNlbGVjdGlvbiA9PT0gXCJmdW5jdGlvblwiID8gc2VsZWN0aW9uLmFwcGx5KHRoaXMsIGFyZ3VtZW50cykgOiBzZWxlY3Rpb24sIHN0YXRlLmV4dGVudCksXG4gICAgICAgICAgICAgICAgaSA9IGludGVycG9sYXRlKHNlbGVjdGlvbjAsIHNlbGVjdGlvbjEpO1xuXG4gICAgICAgICAgICBmdW5jdGlvbiB0d2Vlbih0KSB7XG4gICAgICAgICAgICAgIHN0YXRlLnNlbGVjdGlvbiA9IHQgPT09IDEgJiYgc2VsZWN0aW9uMSA9PT0gbnVsbCA/IG51bGwgOiBpKHQpO1xuICAgICAgICAgICAgICByZWRyYXcuY2FsbCh0aGF0KTtcbiAgICAgICAgICAgICAgZW1pdC5icnVzaCgpO1xuICAgICAgICAgICAgfVxuXG4gICAgICAgICAgICByZXR1cm4gc2VsZWN0aW9uMCAhPT0gbnVsbCAmJiBzZWxlY3Rpb24xICE9PSBudWxsID8gdHdlZW4gOiB0d2VlbigxKTtcbiAgICAgICAgICB9KTtcbiAgICB9IGVsc2Uge1xuICAgICAgZ3JvdXBcbiAgICAgICAgICAuZWFjaChmdW5jdGlvbigpIHtcbiAgICAgICAgICAgIHZhciB0aGF0ID0gdGhpcyxcbiAgICAgICAgICAgICAgICBhcmdzID0gYXJndW1lbnRzLFxuICAgICAgICAgICAgICAgIHN0YXRlID0gdGhhdC5fX2JydXNoLFxuICAgICAgICAgICAgICAgIHNlbGVjdGlvbjEgPSBkaW0uaW5wdXQodHlwZW9mIHNlbGVjdGlvbiA9PT0gXCJmdW5jdGlvblwiID8gc2VsZWN0aW9uLmFwcGx5KHRoYXQsIGFyZ3MpIDogc2VsZWN0aW9uLCBzdGF0ZS5leHRlbnQpLFxuICAgICAgICAgICAgICAgIGVtaXQgPSBlbWl0dGVyKHRoYXQsIGFyZ3MpLmJlZm9yZXN0YXJ0KCk7XG5cbiAgICAgICAgICAgIGludGVycnVwdCh0aGF0KTtcbiAgICAgICAgICAgIHN0YXRlLnNlbGVjdGlvbiA9IHNlbGVjdGlvbjEgPT09IG51bGwgPyBudWxsIDogc2VsZWN0aW9uMTtcbiAgICAgICAgICAgIHJlZHJhdy5jYWxsKHRoYXQpO1xuICAgICAgICAgICAgZW1pdC5zdGFydChldmVudCkuYnJ1c2goZXZlbnQpLmVuZChldmVudCk7XG4gICAgICAgICAgfSk7XG4gICAgfVxuICB9O1xuXG4gIGJydXNoLmNsZWFyID0gZnVuY3Rpb24oZ3JvdXAsIGV2ZW50KSB7XG4gICAgYnJ1c2gubW92ZShncm91cCwgbnVsbCwgZXZlbnQpO1xuICB9O1xuXG4gIGZ1bmN0aW9uIHJlZHJhdygpIHtcbiAgICB2YXIgZ3JvdXAgPSBzZWxlY3QodGhpcyksXG4gICAgICAgIHNlbGVjdGlvbiA9IGxvY2FsKHRoaXMpLnNlbGVjdGlvbjtcblxuICAgIGlmIChzZWxlY3Rpb24pIHtcbiAgICAgIGdyb3VwLnNlbGVjdEFsbChcIi5zZWxlY3Rpb25cIilcbiAgICAgICAgICAuc3R5bGUoXCJkaXNwbGF5XCIsIG51bGwpXG4gICAgICAgICAgLmF0dHIoXCJ4XCIsIHNlbGVjdGlvblswXVswXSlcbiAgICAgICAgICAuYXR0cihcInlcIiwgc2VsZWN0aW9uWzBdWzFdKVxuICAgICAgICAgIC5hdHRyKFwid2lkdGhcIiwgc2VsZWN0aW9uWzFdWzBdIC0gc2VsZWN0aW9uWzBdWzBdKVxuICAgICAgICAgIC5hdHRyKFwiaGVpZ2h0XCIsIHNlbGVjdGlvblsxXVsxXSAtIHNlbGVjdGlvblswXVsxXSk7XG5cbiAgICAgIGdyb3VwLnNlbGVjdEFsbChcIi5oYW5kbGVcIilcbiAgICAgICAgICAuc3R5bGUoXCJkaXNwbGF5XCIsIG51bGwpXG4gICAgICAgICAgLmF0dHIoXCJ4XCIsIGZ1bmN0aW9uKGQpIHsgcmV0dXJuIGQudHlwZVtkLnR5cGUubGVuZ3RoIC0gMV0gPT09IFwiZVwiID8gc2VsZWN0aW9uWzFdWzBdIC0gaGFuZGxlU2l6ZSAvIDIgOiBzZWxlY3Rpb25bMF1bMF0gLSBoYW5kbGVTaXplIC8gMjsgfSlcbiAgICAgICAgICAuYXR0cihcInlcIiwgZnVuY3Rpb24oZCkgeyByZXR1cm4gZC50eXBlWzBdID09PSBcInNcIiA/IHNlbGVjdGlvblsxXVsxXSAtIGhhbmRsZVNpemUgLyAyIDogc2VsZWN0aW9uWzBdWzFdIC0gaGFuZGxlU2l6ZSAvIDI7IH0pXG4gICAgICAgICAgLmF0dHIoXCJ3aWR0aFwiLCBmdW5jdGlvbihkKSB7IHJldHVybiBkLnR5cGUgPT09IFwiblwiIHx8IGQudHlwZSA9PT0gXCJzXCIgPyBzZWxlY3Rpb25bMV1bMF0gLSBzZWxlY3Rpb25bMF1bMF0gKyBoYW5kbGVTaXplIDogaGFuZGxlU2l6ZTsgfSlcbiAgICAgICAgICAuYXR0cihcImhlaWdodFwiLCBmdW5jdGlvbihkKSB7IHJldHVybiBkLnR5cGUgPT09IFwiZVwiIHx8IGQudHlwZSA9PT0gXCJ3XCIgPyBzZWxlY3Rpb25bMV1bMV0gLSBzZWxlY3Rpb25bMF1bMV0gKyBoYW5kbGVTaXplIDogaGFuZGxlU2l6ZTsgfSk7XG4gICAgfVxuXG4gICAgZWxzZSB7XG4gICAgICBncm91cC5zZWxlY3RBbGwoXCIuc2VsZWN0aW9uLC5oYW5kbGVcIilcbiAgICAgICAgICAuc3R5bGUoXCJkaXNwbGF5XCIsIFwibm9uZVwiKVxuICAgICAgICAgIC5hdHRyKFwieFwiLCBudWxsKVxuICAgICAgICAgIC5hdHRyKFwieVwiLCBudWxsKVxuICAgICAgICAgIC5hdHRyKFwid2lkdGhcIiwgbnVsbClcbiAgICAgICAgICAuYXR0cihcImhlaWdodFwiLCBudWxsKTtcbiAgICB9XG4gIH1cblxuICBmdW5jdGlvbiBlbWl0dGVyKHRoYXQsIGFyZ3MsIGNsZWFuKSB7XG4gICAgdmFyIGVtaXQgPSB0aGF0Ll9fYnJ1c2guZW1pdHRlcjtcbiAgICByZXR1cm4gZW1pdCAmJiAoIWNsZWFuIHx8ICFlbWl0LmNsZWFuKSA/IGVtaXQgOiBuZXcgRW1pdHRlcih0aGF0LCBhcmdzLCBjbGVhbik7XG4gIH1cblxuICBmdW5jdGlvbiBFbWl0dGVyKHRoYXQsIGFyZ3MsIGNsZWFuKSB7XG4gICAgdGhpcy50aGF0ID0gdGhhdDtcbiAgICB0aGlzLmFyZ3MgPSBhcmdzO1xuICAgIHRoaXMuc3RhdGUgPSB0aGF0Ll9fYnJ1c2g7XG4gICAgdGhpcy5hY3RpdmUgPSAwO1xuICAgIHRoaXMuY2xlYW4gPSBjbGVhbjtcbiAgfVxuXG4gIEVtaXR0ZXIucHJvdG90eXBlID0ge1xuICAgIGJlZm9yZXN0YXJ0OiBmdW5jdGlvbigpIHtcbiAgICAgIGlmICgrK3RoaXMuYWN0aXZlID09PSAxKSB0aGlzLnN0YXRlLmVtaXR0ZXIgPSB0aGlzLCB0aGlzLnN0YXJ0aW5nID0gdHJ1ZTtcbiAgICAgIHJldHVybiB0aGlzO1xuICAgIH0sXG4gICAgc3RhcnQ6IGZ1bmN0aW9uKGV2ZW50LCBtb2RlKSB7XG4gICAgICBpZiAodGhpcy5zdGFydGluZykgdGhpcy5zdGFydGluZyA9IGZhbHNlLCB0aGlzLmVtaXQoXCJzdGFydFwiLCBldmVudCwgbW9kZSk7XG4gICAgICBlbHNlIHRoaXMuZW1pdChcImJydXNoXCIsIGV2ZW50KTtcbiAgICAgIHJldHVybiB0aGlzO1xuICAgIH0sXG4gICAgYnJ1c2g6IGZ1bmN0aW9uKGV2ZW50LCBtb2RlKSB7XG4gICAgICB0aGlzLmVtaXQoXCJicnVzaFwiLCBldmVudCwgbW9kZSk7XG4gICAgICByZXR1cm4gdGhpcztcbiAgICB9LFxuICAgIGVuZDogZnVuY3Rpb24oZXZlbnQsIG1vZGUpIHtcbiAgICAgIGlmICgtLXRoaXMuYWN0aXZlID09PSAwKSBkZWxldGUgdGhpcy5zdGF0ZS5lbWl0dGVyLCB0aGlzLmVtaXQoXCJlbmRcIiwgZXZlbnQsIG1vZGUpO1xuICAgICAgcmV0dXJuIHRoaXM7XG4gICAgfSxcbiAgICBlbWl0OiBmdW5jdGlvbih0eXBlLCBldmVudCwgbW9kZSkge1xuICAgICAgdmFyIGQgPSBzZWxlY3QodGhpcy50aGF0KS5kYXR1bSgpO1xuICAgICAgbGlzdGVuZXJzLmNhbGwoXG4gICAgICAgIHR5cGUsXG4gICAgICAgIHRoaXMudGhhdCxcbiAgICAgICAgbmV3IEJydXNoRXZlbnQodHlwZSwge1xuICAgICAgICAgIHNvdXJjZUV2ZW50OiBldmVudCxcbiAgICAgICAgICB0YXJnZXQ6IGJydXNoLFxuICAgICAgICAgIHNlbGVjdGlvbjogZGltLm91dHB1dCh0aGlzLnN0YXRlLnNlbGVjdGlvbiksXG4gICAgICAgICAgbW9kZSxcbiAgICAgICAgICBkaXNwYXRjaDogbGlzdGVuZXJzXG4gICAgICAgIH0pLFxuICAgICAgICBkXG4gICAgICApO1xuICAgIH1cbiAgfTtcblxuICBmdW5jdGlvbiBzdGFydGVkKGV2ZW50KSB7XG4gICAgaWYgKHRvdWNoZW5kaW5nICYmICFldmVudC50b3VjaGVzKSByZXR1cm47XG4gICAgaWYgKCFmaWx0ZXIuYXBwbHkodGhpcywgYXJndW1lbnRzKSkgcmV0dXJuO1xuXG4gICAgdmFyIHRoYXQgPSB0aGlzLFxuICAgICAgICB0eXBlID0gZXZlbnQudGFyZ2V0Ll9fZGF0YV9fLnR5cGUsXG4gICAgICAgIG1vZGUgPSAoa2V5cyAmJiBldmVudC5tZXRhS2V5ID8gdHlwZSA9IFwib3ZlcmxheVwiIDogdHlwZSkgPT09IFwic2VsZWN0aW9uXCIgPyBNT0RFX0RSQUcgOiAoa2V5cyAmJiBldmVudC5hbHRLZXkgPyBNT0RFX0NFTlRFUiA6IE1PREVfSEFORExFKSxcbiAgICAgICAgc2lnblggPSBkaW0gPT09IFkgPyBudWxsIDogc2lnbnNYW3R5cGVdLFxuICAgICAgICBzaWduWSA9IGRpbSA9PT0gWCA/IG51bGwgOiBzaWduc1lbdHlwZV0sXG4gICAgICAgIHN0YXRlID0gbG9jYWwodGhhdCksXG4gICAgICAgIGV4dGVudCA9IHN0YXRlLmV4dGVudCxcbiAgICAgICAgc2VsZWN0aW9uID0gc3RhdGUuc2VsZWN0aW9uLFxuICAgICAgICBXID0gZXh0ZW50WzBdWzBdLCB3MCwgdzEsXG4gICAgICAgIE4gPSBleHRlbnRbMF1bMV0sIG4wLCBuMSxcbiAgICAgICAgRSA9IGV4dGVudFsxXVswXSwgZTAsIGUxLFxuICAgICAgICBTID0gZXh0ZW50WzFdWzFdLCBzMCwgczEsXG4gICAgICAgIGR4ID0gMCxcbiAgICAgICAgZHkgPSAwLFxuICAgICAgICBtb3ZpbmcsXG4gICAgICAgIHNoaWZ0aW5nID0gc2lnblggJiYgc2lnblkgJiYga2V5cyAmJiBldmVudC5zaGlmdEtleSxcbiAgICAgICAgbG9ja1gsXG4gICAgICAgIGxvY2tZLFxuICAgICAgICBwb2ludHMgPSBBcnJheS5mcm9tKGV2ZW50LnRvdWNoZXMgfHwgW2V2ZW50XSwgdCA9PiB7XG4gICAgICAgICAgY29uc3QgaSA9IHQuaWRlbnRpZmllcjtcbiAgICAgICAgICB0ID0gcG9pbnRlcih0LCB0aGF0KTtcbiAgICAgICAgICB0LnBvaW50MCA9IHQuc2xpY2UoKTtcbiAgICAgICAgICB0LmlkZW50aWZpZXIgPSBpO1xuICAgICAgICAgIHJldHVybiB0O1xuICAgICAgICB9KTtcblxuICAgIGludGVycnVwdCh0aGF0KTtcbiAgICB2YXIgZW1pdCA9IGVtaXR0ZXIodGhhdCwgYXJndW1lbnRzLCB0cnVlKS5iZWZvcmVzdGFydCgpO1xuXG4gICAgaWYgKHR5cGUgPT09IFwib3ZlcmxheVwiKSB7XG4gICAgICBpZiAoc2VsZWN0aW9uKSBtb3ZpbmcgPSB0cnVlO1xuICAgICAgY29uc3QgcHRzID0gW3BvaW50c1swXSwgcG9pbnRzWzFdIHx8IHBvaW50c1swXV07XG4gICAgICBzdGF0ZS5zZWxlY3Rpb24gPSBzZWxlY3Rpb24gPSBbW1xuICAgICAgICAgIHcwID0gZGltID09PSBZID8gVyA6IG1pbihwdHNbMF1bMF0sIHB0c1sxXVswXSksXG4gICAgICAgICAgbjAgPSBkaW0gPT09IFggPyBOIDogbWluKHB0c1swXVsxXSwgcHRzWzFdWzFdKVxuICAgICAgICBdLCBbXG4gICAgICAgICAgZTAgPSBkaW0gPT09IFkgPyBFIDogbWF4KHB0c1swXVswXSwgcHRzWzFdWzBdKSxcbiAgICAgICAgICBzMCA9IGRpbSA9PT0gWCA/IFMgOiBtYXgocHRzWzBdWzFdLCBwdHNbMV1bMV0pXG4gICAgICAgIF1dO1xuICAgICAgaWYgKHBvaW50cy5sZW5ndGggPiAxKSBtb3ZlKGV2ZW50KTtcbiAgICB9IGVsc2Uge1xuICAgICAgdzAgPSBzZWxlY3Rpb25bMF1bMF07XG4gICAgICBuMCA9IHNlbGVjdGlvblswXVsxXTtcbiAgICAgIGUwID0gc2VsZWN0aW9uWzFdWzBdO1xuICAgICAgczAgPSBzZWxlY3Rpb25bMV1bMV07XG4gICAgfVxuXG4gICAgdzEgPSB3MDtcbiAgICBuMSA9IG4wO1xuICAgIGUxID0gZTA7XG4gICAgczEgPSBzMDtcblxuICAgIHZhciBncm91cCA9IHNlbGVjdCh0aGF0KVxuICAgICAgICAuYXR0cihcInBvaW50ZXItZXZlbnRzXCIsIFwibm9uZVwiKTtcblxuICAgIHZhciBvdmVybGF5ID0gZ3JvdXAuc2VsZWN0QWxsKFwiLm92ZXJsYXlcIilcbiAgICAgICAgLmF0dHIoXCJjdXJzb3JcIiwgY3Vyc29yc1t0eXBlXSk7XG5cbiAgICBpZiAoZXZlbnQudG91Y2hlcykge1xuICAgICAgZW1pdC5tb3ZlZCA9IG1vdmVkO1xuICAgICAgZW1pdC5lbmRlZCA9IGVuZGVkO1xuICAgIH0gZWxzZSB7XG4gICAgICB2YXIgdmlldyA9IHNlbGVjdChldmVudC52aWV3KVxuICAgICAgICAgIC5vbihcIm1vdXNlbW92ZS5icnVzaFwiLCBtb3ZlZCwgdHJ1ZSlcbiAgICAgICAgICAub24oXCJtb3VzZXVwLmJydXNoXCIsIGVuZGVkLCB0cnVlKTtcbiAgICAgIGlmIChrZXlzKSB2aWV3XG4gICAgICAgICAgLm9uKFwia2V5ZG93bi5icnVzaFwiLCBrZXlkb3duZWQsIHRydWUpXG4gICAgICAgICAgLm9uKFwia2V5dXAuYnJ1c2hcIiwga2V5dXBwZWQsIHRydWUpXG5cbiAgICAgIGRyYWdEaXNhYmxlKGV2ZW50LnZpZXcpO1xuICAgIH1cblxuICAgIHJlZHJhdy5jYWxsKHRoYXQpO1xuICAgIGVtaXQuc3RhcnQoZXZlbnQsIG1vZGUubmFtZSk7XG5cbiAgICBmdW5jdGlvbiBtb3ZlZChldmVudCkge1xuICAgICAgZm9yIChjb25zdCBwIG9mIGV2ZW50LmNoYW5nZWRUb3VjaGVzIHx8IFtldmVudF0pIHtcbiAgICAgICAgZm9yIChjb25zdCBkIG9mIHBvaW50cylcbiAgICAgICAgICBpZiAoZC5pZGVudGlmaWVyID09PSBwLmlkZW50aWZpZXIpIGQuY3VyID0gcG9pbnRlcihwLCB0aGF0KTtcbiAgICAgIH1cbiAgICAgIGlmIChzaGlmdGluZyAmJiAhbG9ja1ggJiYgIWxvY2tZICYmIHBvaW50cy5sZW5ndGggPT09IDEpIHtcbiAgICAgICAgY29uc3QgcG9pbnQgPSBwb2ludHNbMF07XG4gICAgICAgIGlmIChhYnMocG9pbnQuY3VyWzBdIC0gcG9pbnRbMF0pID4gYWJzKHBvaW50LmN1clsxXSAtIHBvaW50WzFdKSlcbiAgICAgICAgICBsb2NrWSA9IHRydWU7XG4gICAgICAgIGVsc2VcbiAgICAgICAgICBsb2NrWCA9IHRydWU7XG4gICAgICB9XG4gICAgICBmb3IgKGNvbnN0IHBvaW50IG9mIHBvaW50cylcbiAgICAgICAgaWYgKHBvaW50LmN1cikgcG9pbnRbMF0gPSBwb2ludC5jdXJbMF0sIHBvaW50WzFdID0gcG9pbnQuY3VyWzFdO1xuICAgICAgbW92aW5nID0gdHJ1ZTtcbiAgICAgIG5vZXZlbnQoZXZlbnQpO1xuICAgICAgbW92ZShldmVudCk7XG4gICAgfVxuXG4gICAgZnVuY3Rpb24gbW92ZShldmVudCkge1xuICAgICAgY29uc3QgcG9pbnQgPSBwb2ludHNbMF0sIHBvaW50MCA9IHBvaW50LnBvaW50MDtcbiAgICAgIHZhciB0O1xuXG4gICAgICBkeCA9IHBvaW50WzBdIC0gcG9pbnQwWzBdO1xuICAgICAgZHkgPSBwb2ludFsxXSAtIHBvaW50MFsxXTtcblxuICAgICAgc3dpdGNoIChtb2RlKSB7XG4gICAgICAgIGNhc2UgTU9ERV9TUEFDRTpcbiAgICAgICAgY2FzZSBNT0RFX0RSQUc6IHtcbiAgICAgICAgICBpZiAoc2lnblgpIGR4ID0gbWF4KFcgLSB3MCwgbWluKEUgLSBlMCwgZHgpKSwgdzEgPSB3MCArIGR4LCBlMSA9IGUwICsgZHg7XG4gICAgICAgICAgaWYgKHNpZ25ZKSBkeSA9IG1heChOIC0gbjAsIG1pbihTIC0gczAsIGR5KSksIG4xID0gbjAgKyBkeSwgczEgPSBzMCArIGR5O1xuICAgICAgICAgIGJyZWFrO1xuICAgICAgICB9XG4gICAgICAgIGNhc2UgTU9ERV9IQU5ETEU6IHtcbiAgICAgICAgICBpZiAocG9pbnRzWzFdKSB7XG4gICAgICAgICAgICBpZiAoc2lnblgpIHcxID0gbWF4KFcsIG1pbihFLCBwb2ludHNbMF1bMF0pKSwgZTEgPSBtYXgoVywgbWluKEUsIHBvaW50c1sxXVswXSkpLCBzaWduWCA9IDE7XG4gICAgICAgICAgICBpZiAoc2lnblkpIG4xID0gbWF4KE4sIG1pbihTLCBwb2ludHNbMF1bMV0pKSwgczEgPSBtYXgoTiwgbWluKFMsIHBvaW50c1sxXVsxXSkpLCBzaWduWSA9IDE7XG4gICAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICAgIGlmIChzaWduWCA8IDApIGR4ID0gbWF4KFcgLSB3MCwgbWluKEUgLSB3MCwgZHgpKSwgdzEgPSB3MCArIGR4LCBlMSA9IGUwO1xuICAgICAgICAgICAgZWxzZSBpZiAoc2lnblggPiAwKSBkeCA9IG1heChXIC0gZTAsIG1pbihFIC0gZTAsIGR4KSksIHcxID0gdzAsIGUxID0gZTAgKyBkeDtcbiAgICAgICAgICAgIGlmIChzaWduWSA8IDApIGR5ID0gbWF4KE4gLSBuMCwgbWluKFMgLSBuMCwgZHkpKSwgbjEgPSBuMCArIGR5LCBzMSA9IHMwO1xuICAgICAgICAgICAgZWxzZSBpZiAoc2lnblkgPiAwKSBkeSA9IG1heChOIC0gczAsIG1pbihTIC0gczAsIGR5KSksIG4xID0gbjAsIHMxID0gczAgKyBkeTtcbiAgICAgICAgICB9XG4gICAgICAgICAgYnJlYWs7XG4gICAgICAgIH1cbiAgICAgICAgY2FzZSBNT0RFX0NFTlRFUjoge1xuICAgICAgICAgIGlmIChzaWduWCkgdzEgPSBtYXgoVywgbWluKEUsIHcwIC0gZHggKiBzaWduWCkpLCBlMSA9IG1heChXLCBtaW4oRSwgZTAgKyBkeCAqIHNpZ25YKSk7XG4gICAgICAgICAgaWYgKHNpZ25ZKSBuMSA9IG1heChOLCBtaW4oUywgbjAgLSBkeSAqIHNpZ25ZKSksIHMxID0gbWF4KE4sIG1pbihTLCBzMCArIGR5ICogc2lnblkpKTtcbiAgICAgICAgICBicmVhaztcbiAgICAgICAgfVxuICAgICAgfVxuXG4gICAgICBpZiAoZTEgPCB3MSkge1xuICAgICAgICBzaWduWCAqPSAtMTtcbiAgICAgICAgdCA9IHcwLCB3MCA9IGUwLCBlMCA9IHQ7XG4gICAgICAgIHQgPSB3MSwgdzEgPSBlMSwgZTEgPSB0O1xuICAgICAgICBpZiAodHlwZSBpbiBmbGlwWCkgb3ZlcmxheS5hdHRyKFwiY3Vyc29yXCIsIGN1cnNvcnNbdHlwZSA9IGZsaXBYW3R5cGVdXSk7XG4gICAgICB9XG5cbiAgICAgIGlmIChzMSA8IG4xKSB7XG4gICAgICAgIHNpZ25ZICo9IC0xO1xuICAgICAgICB0ID0gbjAsIG4wID0gczAsIHMwID0gdDtcbiAgICAgICAgdCA9IG4xLCBuMSA9IHMxLCBzMSA9IHQ7XG4gICAgICAgIGlmICh0eXBlIGluIGZsaXBZKSBvdmVybGF5LmF0dHIoXCJjdXJzb3JcIiwgY3Vyc29yc1t0eXBlID0gZmxpcFlbdHlwZV1dKTtcbiAgICAgIH1cblxuICAgICAgaWYgKHN0YXRlLnNlbGVjdGlvbikgc2VsZWN0aW9uID0gc3RhdGUuc2VsZWN0aW9uOyAvLyBNYXkgYmUgc2V0IGJ5IGJydXNoLm1vdmUhXG4gICAgICBpZiAobG9ja1gpIHcxID0gc2VsZWN0aW9uWzBdWzBdLCBlMSA9IHNlbGVjdGlvblsxXVswXTtcbiAgICAgIGlmIChsb2NrWSkgbjEgPSBzZWxlY3Rpb25bMF1bMV0sIHMxID0gc2VsZWN0aW9uWzFdWzFdO1xuXG4gICAgICBpZiAoc2VsZWN0aW9uWzBdWzBdICE9PSB3MVxuICAgICAgICAgIHx8IHNlbGVjdGlvblswXVsxXSAhPT0gbjFcbiAgICAgICAgICB8fCBzZWxlY3Rpb25bMV1bMF0gIT09IGUxXG4gICAgICAgICAgfHwgc2VsZWN0aW9uWzFdWzFdICE9PSBzMSkge1xuICAgICAgICBzdGF0ZS5zZWxlY3Rpb24gPSBbW3cxLCBuMV0sIFtlMSwgczFdXTtcbiAgICAgICAgcmVkcmF3LmNhbGwodGhhdCk7XG4gICAgICAgIGVtaXQuYnJ1c2goZXZlbnQsIG1vZGUubmFtZSk7XG4gICAgICB9XG4gICAgfVxuXG4gICAgZnVuY3Rpb24gZW5kZWQoZXZlbnQpIHtcbiAgICAgIG5vcHJvcGFnYXRpb24oZXZlbnQpO1xuICAgICAgaWYgKGV2ZW50LnRvdWNoZXMpIHtcbiAgICAgICAgaWYgKGV2ZW50LnRvdWNoZXMubGVuZ3RoKSByZXR1cm47XG4gICAgICAgIGlmICh0b3VjaGVuZGluZykgY2xlYXJUaW1lb3V0KHRvdWNoZW5kaW5nKTtcbiAgICAgICAgdG91Y2hlbmRpbmcgPSBzZXRUaW1lb3V0KGZ1bmN0aW9uKCkgeyB0b3VjaGVuZGluZyA9IG51bGw7IH0sIDUwMCk7IC8vIEdob3N0IGNsaWNrcyBhcmUgZGVsYXllZCFcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIGRyYWdFbmFibGUoZXZlbnQudmlldywgbW92aW5nKTtcbiAgICAgICAgdmlldy5vbihcImtleWRvd24uYnJ1c2gga2V5dXAuYnJ1c2ggbW91c2Vtb3ZlLmJydXNoIG1vdXNldXAuYnJ1c2hcIiwgbnVsbCk7XG4gICAgICB9XG4gICAgICBncm91cC5hdHRyKFwicG9pbnRlci1ldmVudHNcIiwgXCJhbGxcIik7XG4gICAgICBvdmVybGF5LmF0dHIoXCJjdXJzb3JcIiwgY3Vyc29ycy5vdmVybGF5KTtcbiAgICAgIGlmIChzdGF0ZS5zZWxlY3Rpb24pIHNlbGVjdGlvbiA9IHN0YXRlLnNlbGVjdGlvbjsgLy8gTWF5IGJlIHNldCBieSBicnVzaC5tb3ZlIChvbiBzdGFydCkhXG4gICAgICBpZiAoZW1wdHkoc2VsZWN0aW9uKSkgc3RhdGUuc2VsZWN0aW9uID0gbnVsbCwgcmVkcmF3LmNhbGwodGhhdCk7XG4gICAgICBlbWl0LmVuZChldmVudCwgbW9kZS5uYW1lKTtcbiAgICB9XG5cbiAgICBmdW5jdGlvbiBrZXlkb3duZWQoZXZlbnQpIHtcbiAgICAgIHN3aXRjaCAoZXZlbnQua2V5Q29kZSkge1xuICAgICAgICBjYXNlIDE2OiB7IC8vIFNISUZUXG4gICAgICAgICAgc2hpZnRpbmcgPSBzaWduWCAmJiBzaWduWTtcbiAgICAgICAgICBicmVhaztcbiAgICAgICAgfVxuICAgICAgICBjYXNlIDE4OiB7IC8vIEFMVFxuICAgICAgICAgIGlmIChtb2RlID09PSBNT0RFX0hBTkRMRSkge1xuICAgICAgICAgICAgaWYgKHNpZ25YKSBlMCA9IGUxIC0gZHggKiBzaWduWCwgdzAgPSB3MSArIGR4ICogc2lnblg7XG4gICAgICAgICAgICBpZiAoc2lnblkpIHMwID0gczEgLSBkeSAqIHNpZ25ZLCBuMCA9IG4xICsgZHkgKiBzaWduWTtcbiAgICAgICAgICAgIG1vZGUgPSBNT0RFX0NFTlRFUjtcbiAgICAgICAgICAgIG1vdmUoZXZlbnQpO1xuICAgICAgICAgIH1cbiAgICAgICAgICBicmVhaztcbiAgICAgICAgfVxuICAgICAgICBjYXNlIDMyOiB7IC8vIFNQQUNFOyB0YWtlcyBwcmlvcml0eSBvdmVyIEFMVFxuICAgICAgICAgIGlmIChtb2RlID09PSBNT0RFX0hBTkRMRSB8fCBtb2RlID09PSBNT0RFX0NFTlRFUikge1xuICAgICAgICAgICAgaWYgKHNpZ25YIDwgMCkgZTAgPSBlMSAtIGR4OyBlbHNlIGlmIChzaWduWCA+IDApIHcwID0gdzEgLSBkeDtcbiAgICAgICAgICAgIGlmIChzaWduWSA8IDApIHMwID0gczEgLSBkeTsgZWxzZSBpZiAoc2lnblkgPiAwKSBuMCA9IG4xIC0gZHk7XG4gICAgICAgICAgICBtb2RlID0gTU9ERV9TUEFDRTtcbiAgICAgICAgICAgIG92ZXJsYXkuYXR0cihcImN1cnNvclwiLCBjdXJzb3JzLnNlbGVjdGlvbik7XG4gICAgICAgICAgICBtb3ZlKGV2ZW50KTtcbiAgICAgICAgICB9XG4gICAgICAgICAgYnJlYWs7XG4gICAgICAgIH1cbiAgICAgICAgZGVmYXVsdDogcmV0dXJuO1xuICAgICAgfVxuICAgICAgbm9ldmVudChldmVudCk7XG4gICAgfVxuXG4gICAgZnVuY3Rpb24ga2V5dXBwZWQoZXZlbnQpIHtcbiAgICAgIHN3aXRjaCAoZXZlbnQua2V5Q29kZSkge1xuICAgICAgICBjYXNlIDE2OiB7IC8vIFNISUZUXG4gICAgICAgICAgaWYgKHNoaWZ0aW5nKSB7XG4gICAgICAgICAgICBsb2NrWCA9IGxvY2tZID0gc2hpZnRpbmcgPSBmYWxzZTtcbiAgICAgICAgICAgIG1vdmUoZXZlbnQpO1xuICAgICAgICAgIH1cbiAgICAgICAgICBicmVhaztcbiAgICAgICAgfVxuICAgICAgICBjYXNlIDE4OiB7IC8vIEFMVFxuICAgICAgICAgIGlmIChtb2RlID09PSBNT0RFX0NFTlRFUikge1xuICAgICAgICAgICAgaWYgKHNpZ25YIDwgMCkgZTAgPSBlMTsgZWxzZSBpZiAoc2lnblggPiAwKSB3MCA9IHcxO1xuICAgICAgICAgICAgaWYgKHNpZ25ZIDwgMCkgczAgPSBzMTsgZWxzZSBpZiAoc2lnblkgPiAwKSBuMCA9IG4xO1xuICAgICAgICAgICAgbW9kZSA9IE1PREVfSEFORExFO1xuICAgICAgICAgICAgbW92ZShldmVudCk7XG4gICAgICAgICAgfVxuICAgICAgICAgIGJyZWFrO1xuICAgICAgICB9XG4gICAgICAgIGNhc2UgMzI6IHsgLy8gU1BBQ0VcbiAgICAgICAgICBpZiAobW9kZSA9PT0gTU9ERV9TUEFDRSkge1xuICAgICAgICAgICAgaWYgKGV2ZW50LmFsdEtleSkge1xuICAgICAgICAgICAgICBpZiAoc2lnblgpIGUwID0gZTEgLSBkeCAqIHNpZ25YLCB3MCA9IHcxICsgZHggKiBzaWduWDtcbiAgICAgICAgICAgICAgaWYgKHNpZ25ZKSBzMCA9IHMxIC0gZHkgKiBzaWduWSwgbjAgPSBuMSArIGR5ICogc2lnblk7XG4gICAgICAgICAgICAgIG1vZGUgPSBNT0RFX0NFTlRFUjtcbiAgICAgICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgICAgIGlmIChzaWduWCA8IDApIGUwID0gZTE7IGVsc2UgaWYgKHNpZ25YID4gMCkgdzAgPSB3MTtcbiAgICAgICAgICAgICAgaWYgKHNpZ25ZIDwgMCkgczAgPSBzMTsgZWxzZSBpZiAoc2lnblkgPiAwKSBuMCA9IG4xO1xuICAgICAgICAgICAgICBtb2RlID0gTU9ERV9IQU5ETEU7XG4gICAgICAgICAgICB9XG4gICAgICAgICAgICBvdmVybGF5LmF0dHIoXCJjdXJzb3JcIiwgY3Vyc29yc1t0eXBlXSk7XG4gICAgICAgICAgICBtb3ZlKGV2ZW50KTtcbiAgICAgICAgICB9XG4gICAgICAgICAgYnJlYWs7XG4gICAgICAgIH1cbiAgICAgICAgZGVmYXVsdDogcmV0dXJuO1xuICAgICAgfVxuICAgICAgbm9ldmVudChldmVudCk7XG4gICAgfVxuICB9XG5cbiAgZnVuY3Rpb24gdG91Y2htb3ZlZChldmVudCkge1xuICAgIGVtaXR0ZXIodGhpcywgYXJndW1lbnRzKS5tb3ZlZChldmVudCk7XG4gIH1cblxuICBmdW5jdGlvbiB0b3VjaGVuZGVkKGV2ZW50KSB7XG4gICAgZW1pdHRlcih0aGlzLCBhcmd1bWVudHMpLmVuZGVkKGV2ZW50KTtcbiAgfVxuXG4gIGZ1bmN0aW9uIGluaXRpYWxpemUoKSB7XG4gICAgdmFyIHN0YXRlID0gdGhpcy5fX2JydXNoIHx8IHtzZWxlY3Rpb246IG51bGx9O1xuICAgIHN0YXRlLmV4dGVudCA9IG51bWJlcjIoZXh0ZW50LmFwcGx5KHRoaXMsIGFyZ3VtZW50cykpO1xuICAgIHN0YXRlLmRpbSA9IGRpbTtcbiAgICByZXR1cm4gc3RhdGU7XG4gIH1cblxuICBicnVzaC5leHRlbnQgPSBmdW5jdGlvbihfKSB7XG4gICAgcmV0dXJuIGFyZ3VtZW50cy5sZW5ndGggPyAoZXh0ZW50ID0gdHlwZW9mIF8gPT09IFwiZnVuY3Rpb25cIiA/IF8gOiBjb25zdGFudChudW1iZXIyKF8pKSwgYnJ1c2gpIDogZXh0ZW50O1xuICB9O1xuXG4gIGJydXNoLmZpbHRlciA9IGZ1bmN0aW9uKF8pIHtcbiAgICByZXR1cm4gYXJndW1lbnRzLmxlbmd0aCA/IChmaWx0ZXIgPSB0eXBlb2YgXyA9PT0gXCJmdW5jdGlvblwiID8gXyA6IGNvbnN0YW50KCEhXyksIGJydXNoKSA6IGZpbHRlcjtcbiAgfTtcblxuICBicnVzaC50b3VjaGFibGUgPSBmdW5jdGlvbihfKSB7XG4gICAgcmV0dXJuIGFyZ3VtZW50cy5sZW5ndGggPyAodG91Y2hhYmxlID0gdHlwZW9mIF8gPT09IFwiZnVuY3Rpb25cIiA/IF8gOiBjb25zdGFudCghIV8pLCBicnVzaCkgOiB0b3VjaGFibGU7XG4gIH07XG5cbiAgYnJ1c2guaGFuZGxlU2l6ZSA9IGZ1bmN0aW9uKF8pIHtcbiAgICByZXR1cm4gYXJndW1lbnRzLmxlbmd0aCA/IChoYW5kbGVTaXplID0gK18sIGJydXNoKSA6IGhhbmRsZVNpemU7XG4gIH07XG5cbiAgYnJ1c2gua2V5TW9kaWZpZXJzID0gZnVuY3Rpb24oXykge1xuICAgIHJldHVybiBhcmd1bWVudHMubGVuZ3RoID8gKGtleXMgPSAhIV8sIGJydXNoKSA6IGtleXM7XG4gIH07XG5cbiAgYnJ1c2gub24gPSBmdW5jdGlvbigpIHtcbiAgICB2YXIgdmFsdWUgPSBsaXN0ZW5lcnMub24uYXBwbHkobGlzdGVuZXJzLCBhcmd1bWVudHMpO1xuICAgIHJldHVybiB2YWx1ZSA9PT0gbGlzdGVuZXJzID8gYnJ1c2ggOiB2YWx1ZTtcbiAgfTtcblxuICByZXR1cm4gYnJ1c2g7XG59XG4iLCAiZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24oeCwgeSkge1xuICB2YXIgbm9kZXMsIHN0cmVuZ3RoID0gMTtcblxuICBpZiAoeCA9PSBudWxsKSB4ID0gMDtcbiAgaWYgKHkgPT0gbnVsbCkgeSA9IDA7XG5cbiAgZnVuY3Rpb24gZm9yY2UoKSB7XG4gICAgdmFyIGksXG4gICAgICAgIG4gPSBub2Rlcy5sZW5ndGgsXG4gICAgICAgIG5vZGUsXG4gICAgICAgIHN4ID0gMCxcbiAgICAgICAgc3kgPSAwO1xuXG4gICAgZm9yIChpID0gMDsgaSA8IG47ICsraSkge1xuICAgICAgbm9kZSA9IG5vZGVzW2ldLCBzeCArPSBub2RlLngsIHN5ICs9IG5vZGUueTtcbiAgICB9XG5cbiAgICBmb3IgKHN4ID0gKHN4IC8gbiAtIHgpICogc3RyZW5ndGgsIHN5ID0gKHN5IC8gbiAtIHkpICogc3RyZW5ndGgsIGkgPSAwOyBpIDwgbjsgKytpKSB7XG4gICAgICBub2RlID0gbm9kZXNbaV0sIG5vZGUueCAtPSBzeCwgbm9kZS55IC09IHN5O1xuICAgIH1cbiAgfVxuXG4gIGZvcmNlLmluaXRpYWxpemUgPSBmdW5jdGlvbihfKSB7XG4gICAgbm9kZXMgPSBfO1xuICB9O1xuXG4gIGZvcmNlLnggPSBmdW5jdGlvbihfKSB7XG4gICAgcmV0dXJuIGFyZ3VtZW50cy5sZW5ndGggPyAoeCA9ICtfLCBmb3JjZSkgOiB4O1xuICB9O1xuXG4gIGZvcmNlLnkgPSBmdW5jdGlvbihfKSB7XG4gICAgcmV0dXJuIGFyZ3VtZW50cy5sZW5ndGggPyAoeSA9ICtfLCBmb3JjZSkgOiB5O1xuICB9O1xuXG4gIGZvcmNlLnN0cmVuZ3RoID0gZnVuY3Rpb24oXykge1xuICAgIHJldHVybiBhcmd1bWVudHMubGVuZ3RoID8gKHN0cmVuZ3RoID0gK18sIGZvcmNlKSA6IHN0cmVuZ3RoO1xuICB9O1xuXG4gIHJldHVybiBmb3JjZTtcbn1cbiIsICJleHBvcnQgZGVmYXVsdCBmdW5jdGlvbihkKSB7XG4gIGNvbnN0IHggPSArdGhpcy5feC5jYWxsKG51bGwsIGQpLFxuICAgICAgeSA9ICt0aGlzLl95LmNhbGwobnVsbCwgZCk7XG4gIHJldHVybiBhZGQodGhpcy5jb3Zlcih4LCB5KSwgeCwgeSwgZCk7XG59XG5cbmZ1bmN0aW9uIGFkZCh0cmVlLCB4LCB5LCBkKSB7XG4gIGlmIChpc05hTih4KSB8fCBpc05hTih5KSkgcmV0dXJuIHRyZWU7IC8vIGlnbm9yZSBpbnZhbGlkIHBvaW50c1xuXG4gIHZhciBwYXJlbnQsXG4gICAgICBub2RlID0gdHJlZS5fcm9vdCxcbiAgICAgIGxlYWYgPSB7ZGF0YTogZH0sXG4gICAgICB4MCA9IHRyZWUuX3gwLFxuICAgICAgeTAgPSB0cmVlLl95MCxcbiAgICAgIHgxID0gdHJlZS5feDEsXG4gICAgICB5MSA9IHRyZWUuX3kxLFxuICAgICAgeG0sXG4gICAgICB5bSxcbiAgICAgIHhwLFxuICAgICAgeXAsXG4gICAgICByaWdodCxcbiAgICAgIGJvdHRvbSxcbiAgICAgIGksXG4gICAgICBqO1xuXG4gIC8vIElmIHRoZSB0cmVlIGlzIGVtcHR5LCBpbml0aWFsaXplIHRoZSByb290IGFzIGEgbGVhZi5cbiAgaWYgKCFub2RlKSByZXR1cm4gdHJlZS5fcm9vdCA9IGxlYWYsIHRyZWU7XG5cbiAgLy8gRmluZCB0aGUgZXhpc3RpbmcgbGVhZiBmb3IgdGhlIG5ldyBwb2ludCwgb3IgYWRkIGl0LlxuICB3aGlsZSAobm9kZS5sZW5ndGgpIHtcbiAgICBpZiAocmlnaHQgPSB4ID49ICh4bSA9ICh4MCArIHgxKSAvIDIpKSB4MCA9IHhtOyBlbHNlIHgxID0geG07XG4gICAgaWYgKGJvdHRvbSA9IHkgPj0gKHltID0gKHkwICsgeTEpIC8gMikpIHkwID0geW07IGVsc2UgeTEgPSB5bTtcbiAgICBpZiAocGFyZW50ID0gbm9kZSwgIShub2RlID0gbm9kZVtpID0gYm90dG9tIDw8IDEgfCByaWdodF0pKSByZXR1cm4gcGFyZW50W2ldID0gbGVhZiwgdHJlZTtcbiAgfVxuXG4gIC8vIElzIHRoZSBuZXcgcG9pbnQgaXMgZXhhY3RseSBjb2luY2lkZW50IHdpdGggdGhlIGV4aXN0aW5nIHBvaW50P1xuICB4cCA9ICt0cmVlLl94LmNhbGwobnVsbCwgbm9kZS5kYXRhKTtcbiAgeXAgPSArdHJlZS5feS5jYWxsKG51bGwsIG5vZGUuZGF0YSk7XG4gIGlmICh4ID09PSB4cCAmJiB5ID09PSB5cCkgcmV0dXJuIGxlYWYubmV4dCA9IG5vZGUsIHBhcmVudCA/IHBhcmVudFtpXSA9IGxlYWYgOiB0cmVlLl9yb290ID0gbGVhZiwgdHJlZTtcblxuICAvLyBPdGhlcndpc2UsIHNwbGl0IHRoZSBsZWFmIG5vZGUgdW50aWwgdGhlIG9sZCBhbmQgbmV3IHBvaW50IGFyZSBzZXBhcmF0ZWQuXG4gIGRvIHtcbiAgICBwYXJlbnQgPSBwYXJlbnQgPyBwYXJlbnRbaV0gPSBuZXcgQXJyYXkoNCkgOiB0cmVlLl9yb290ID0gbmV3IEFycmF5KDQpO1xuICAgIGlmIChyaWdodCA9IHggPj0gKHhtID0gKHgwICsgeDEpIC8gMikpIHgwID0geG07IGVsc2UgeDEgPSB4bTtcbiAgICBpZiAoYm90dG9tID0geSA+PSAoeW0gPSAoeTAgKyB5MSkgLyAyKSkgeTAgPSB5bTsgZWxzZSB5MSA9IHltO1xuICB9IHdoaWxlICgoaSA9IGJvdHRvbSA8PCAxIHwgcmlnaHQpID09PSAoaiA9ICh5cCA+PSB5bSkgPDwgMSB8ICh4cCA+PSB4bSkpKTtcbiAgcmV0dXJuIHBhcmVudFtqXSA9IG5vZGUsIHBhcmVudFtpXSA9IGxlYWYsIHRyZWU7XG59XG5cbmV4cG9ydCBmdW5jdGlvbiBhZGRBbGwoZGF0YSkge1xuICB2YXIgZCwgaSwgbiA9IGRhdGEubGVuZ3RoLFxuICAgICAgeCxcbiAgICAgIHksXG4gICAgICB4eiA9IG5ldyBBcnJheShuKSxcbiAgICAgIHl6ID0gbmV3IEFycmF5KG4pLFxuICAgICAgeDAgPSBJbmZpbml0eSxcbiAgICAgIHkwID0gSW5maW5pdHksXG4gICAgICB4MSA9IC1JbmZpbml0eSxcbiAgICAgIHkxID0gLUluZmluaXR5O1xuXG4gIC8vIENvbXB1dGUgdGhlIHBvaW50cyBhbmQgdGhlaXIgZXh0ZW50LlxuICBmb3IgKGkgPSAwOyBpIDwgbjsgKytpKSB7XG4gICAgaWYgKGlzTmFOKHggPSArdGhpcy5feC5jYWxsKG51bGwsIGQgPSBkYXRhW2ldKSkgfHwgaXNOYU4oeSA9ICt0aGlzLl95LmNhbGwobnVsbCwgZCkpKSBjb250aW51ZTtcbiAgICB4eltpXSA9IHg7XG4gICAgeXpbaV0gPSB5O1xuICAgIGlmICh4IDwgeDApIHgwID0geDtcbiAgICBpZiAoeCA+IHgxKSB4MSA9IHg7XG4gICAgaWYgKHkgPCB5MCkgeTAgPSB5O1xuICAgIGlmICh5ID4geTEpIHkxID0geTtcbiAgfVxuXG4gIC8vIElmIHRoZXJlIHdlcmUgbm8gKHZhbGlkKSBwb2ludHMsIGFib3J0LlxuICBpZiAoeDAgPiB4MSB8fCB5MCA+IHkxKSByZXR1cm4gdGhpcztcblxuICAvLyBFeHBhbmQgdGhlIHRyZWUgdG8gY292ZXIgdGhlIG5ldyBwb2ludHMuXG4gIHRoaXMuY292ZXIoeDAsIHkwKS5jb3Zlcih4MSwgeTEpO1xuXG4gIC8vIEFkZCB0aGUgbmV3IHBvaW50cy5cbiAgZm9yIChpID0gMDsgaSA8IG47ICsraSkge1xuICAgIGFkZCh0aGlzLCB4eltpXSwgeXpbaV0sIGRhdGFbaV0pO1xuICB9XG5cbiAgcmV0dXJuIHRoaXM7XG59XG4iLCAiZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24oeCwgeSkge1xuICBpZiAoaXNOYU4oeCA9ICt4KSB8fCBpc05hTih5ID0gK3kpKSByZXR1cm4gdGhpczsgLy8gaWdub3JlIGludmFsaWQgcG9pbnRzXG5cbiAgdmFyIHgwID0gdGhpcy5feDAsXG4gICAgICB5MCA9IHRoaXMuX3kwLFxuICAgICAgeDEgPSB0aGlzLl94MSxcbiAgICAgIHkxID0gdGhpcy5feTE7XG5cbiAgLy8gSWYgdGhlIHF1YWR0cmVlIGhhcyBubyBleHRlbnQsIGluaXRpYWxpemUgdGhlbS5cbiAgLy8gSW50ZWdlciBleHRlbnQgYXJlIG5lY2Vzc2FyeSBzbyB0aGF0IGlmIHdlIGxhdGVyIGRvdWJsZSB0aGUgZXh0ZW50LFxuICAvLyB0aGUgZXhpc3RpbmcgcXVhZHJhbnQgYm91bmRhcmllcyBkb25cdTIwMTl0IGNoYW5nZSBkdWUgdG8gZmxvYXRpbmcgcG9pbnQgZXJyb3IhXG4gIGlmIChpc05hTih4MCkpIHtcbiAgICB4MSA9ICh4MCA9IE1hdGguZmxvb3IoeCkpICsgMTtcbiAgICB5MSA9ICh5MCA9IE1hdGguZmxvb3IoeSkpICsgMTtcbiAgfVxuXG4gIC8vIE90aGVyd2lzZSwgZG91YmxlIHJlcGVhdGVkbHkgdG8gY292ZXIuXG4gIGVsc2Uge1xuICAgIHZhciB6ID0geDEgLSB4MCB8fCAxLFxuICAgICAgICBub2RlID0gdGhpcy5fcm9vdCxcbiAgICAgICAgcGFyZW50LFxuICAgICAgICBpO1xuXG4gICAgd2hpbGUgKHgwID4geCB8fCB4ID49IHgxIHx8IHkwID4geSB8fCB5ID49IHkxKSB7XG4gICAgICBpID0gKHkgPCB5MCkgPDwgMSB8ICh4IDwgeDApO1xuICAgICAgcGFyZW50ID0gbmV3IEFycmF5KDQpLCBwYXJlbnRbaV0gPSBub2RlLCBub2RlID0gcGFyZW50LCB6ICo9IDI7XG4gICAgICBzd2l0Y2ggKGkpIHtcbiAgICAgICAgY2FzZSAwOiB4MSA9IHgwICsgeiwgeTEgPSB5MCArIHo7IGJyZWFrO1xuICAgICAgICBjYXNlIDE6IHgwID0geDEgLSB6LCB5MSA9IHkwICsgejsgYnJlYWs7XG4gICAgICAgIGNhc2UgMjogeDEgPSB4MCArIHosIHkwID0geTEgLSB6OyBicmVhaztcbiAgICAgICAgY2FzZSAzOiB4MCA9IHgxIC0geiwgeTAgPSB5MSAtIHo7IGJyZWFrO1xuICAgICAgfVxuICAgIH1cblxuICAgIGlmICh0aGlzLl9yb290ICYmIHRoaXMuX3Jvb3QubGVuZ3RoKSB0aGlzLl9yb290ID0gbm9kZTtcbiAgfVxuXG4gIHRoaXMuX3gwID0geDA7XG4gIHRoaXMuX3kwID0geTA7XG4gIHRoaXMuX3gxID0geDE7XG4gIHRoaXMuX3kxID0geTE7XG4gIHJldHVybiB0aGlzO1xufVxuIiwgImV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKCkge1xuICB2YXIgZGF0YSA9IFtdO1xuICB0aGlzLnZpc2l0KGZ1bmN0aW9uKG5vZGUpIHtcbiAgICBpZiAoIW5vZGUubGVuZ3RoKSBkbyBkYXRhLnB1c2gobm9kZS5kYXRhKTsgd2hpbGUgKG5vZGUgPSBub2RlLm5leHQpXG4gIH0pO1xuICByZXR1cm4gZGF0YTtcbn1cbiIsICJleHBvcnQgZGVmYXVsdCBmdW5jdGlvbihfKSB7XG4gIHJldHVybiBhcmd1bWVudHMubGVuZ3RoXG4gICAgICA/IHRoaXMuY292ZXIoK19bMF1bMF0sICtfWzBdWzFdKS5jb3ZlcigrX1sxXVswXSwgK19bMV1bMV0pXG4gICAgICA6IGlzTmFOKHRoaXMuX3gwKSA/IHVuZGVmaW5lZCA6IFtbdGhpcy5feDAsIHRoaXMuX3kwXSwgW3RoaXMuX3gxLCB0aGlzLl95MV1dO1xufVxuIiwgImV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKG5vZGUsIHgwLCB5MCwgeDEsIHkxKSB7XG4gIHRoaXMubm9kZSA9IG5vZGU7XG4gIHRoaXMueDAgPSB4MDtcbiAgdGhpcy55MCA9IHkwO1xuICB0aGlzLngxID0geDE7XG4gIHRoaXMueTEgPSB5MTtcbn1cbiIsICJpbXBvcnQgUXVhZCBmcm9tIFwiLi9xdWFkLmpzXCI7XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKHgsIHksIHJhZGl1cykge1xuICB2YXIgZGF0YSxcbiAgICAgIHgwID0gdGhpcy5feDAsXG4gICAgICB5MCA9IHRoaXMuX3kwLFxuICAgICAgeDEsXG4gICAgICB5MSxcbiAgICAgIHgyLFxuICAgICAgeTIsXG4gICAgICB4MyA9IHRoaXMuX3gxLFxuICAgICAgeTMgPSB0aGlzLl95MSxcbiAgICAgIHF1YWRzID0gW10sXG4gICAgICBub2RlID0gdGhpcy5fcm9vdCxcbiAgICAgIHEsXG4gICAgICBpO1xuXG4gIGlmIChub2RlKSBxdWFkcy5wdXNoKG5ldyBRdWFkKG5vZGUsIHgwLCB5MCwgeDMsIHkzKSk7XG4gIGlmIChyYWRpdXMgPT0gbnVsbCkgcmFkaXVzID0gSW5maW5pdHk7XG4gIGVsc2Uge1xuICAgIHgwID0geCAtIHJhZGl1cywgeTAgPSB5IC0gcmFkaXVzO1xuICAgIHgzID0geCArIHJhZGl1cywgeTMgPSB5ICsgcmFkaXVzO1xuICAgIHJhZGl1cyAqPSByYWRpdXM7XG4gIH1cblxuICB3aGlsZSAocSA9IHF1YWRzLnBvcCgpKSB7XG5cbiAgICAvLyBTdG9wIHNlYXJjaGluZyBpZiB0aGlzIHF1YWRyYW50IGNhblx1MjAxOXQgY29udGFpbiBhIGNsb3NlciBub2RlLlxuICAgIGlmICghKG5vZGUgPSBxLm5vZGUpXG4gICAgICAgIHx8ICh4MSA9IHEueDApID4geDNcbiAgICAgICAgfHwgKHkxID0gcS55MCkgPiB5M1xuICAgICAgICB8fCAoeDIgPSBxLngxKSA8IHgwXG4gICAgICAgIHx8ICh5MiA9IHEueTEpIDwgeTApIGNvbnRpbnVlO1xuXG4gICAgLy8gQmlzZWN0IHRoZSBjdXJyZW50IHF1YWRyYW50LlxuICAgIGlmIChub2RlLmxlbmd0aCkge1xuICAgICAgdmFyIHhtID0gKHgxICsgeDIpIC8gMixcbiAgICAgICAgICB5bSA9ICh5MSArIHkyKSAvIDI7XG5cbiAgICAgIHF1YWRzLnB1c2goXG4gICAgICAgIG5ldyBRdWFkKG5vZGVbM10sIHhtLCB5bSwgeDIsIHkyKSxcbiAgICAgICAgbmV3IFF1YWQobm9kZVsyXSwgeDEsIHltLCB4bSwgeTIpLFxuICAgICAgICBuZXcgUXVhZChub2RlWzFdLCB4bSwgeTEsIHgyLCB5bSksXG4gICAgICAgIG5ldyBRdWFkKG5vZGVbMF0sIHgxLCB5MSwgeG0sIHltKVxuICAgICAgKTtcblxuICAgICAgLy8gVmlzaXQgdGhlIGNsb3Nlc3QgcXVhZHJhbnQgZmlyc3QuXG4gICAgICBpZiAoaSA9ICh5ID49IHltKSA8PCAxIHwgKHggPj0geG0pKSB7XG4gICAgICAgIHEgPSBxdWFkc1txdWFkcy5sZW5ndGggLSAxXTtcbiAgICAgICAgcXVhZHNbcXVhZHMubGVuZ3RoIC0gMV0gPSBxdWFkc1txdWFkcy5sZW5ndGggLSAxIC0gaV07XG4gICAgICAgIHF1YWRzW3F1YWRzLmxlbmd0aCAtIDEgLSBpXSA9IHE7XG4gICAgICB9XG4gICAgfVxuXG4gICAgLy8gVmlzaXQgdGhpcyBwb2ludC4gKFZpc2l0aW5nIGNvaW5jaWRlbnQgcG9pbnRzIGlzblx1MjAxOXQgbmVjZXNzYXJ5ISlcbiAgICBlbHNlIHtcbiAgICAgIHZhciBkeCA9IHggLSArdGhpcy5feC5jYWxsKG51bGwsIG5vZGUuZGF0YSksXG4gICAgICAgICAgZHkgPSB5IC0gK3RoaXMuX3kuY2FsbChudWxsLCBub2RlLmRhdGEpLFxuICAgICAgICAgIGQyID0gZHggKiBkeCArIGR5ICogZHk7XG4gICAgICBpZiAoZDIgPCByYWRpdXMpIHtcbiAgICAgICAgdmFyIGQgPSBNYXRoLnNxcnQocmFkaXVzID0gZDIpO1xuICAgICAgICB4MCA9IHggLSBkLCB5MCA9IHkgLSBkO1xuICAgICAgICB4MyA9IHggKyBkLCB5MyA9IHkgKyBkO1xuICAgICAgICBkYXRhID0gbm9kZS5kYXRhO1xuICAgICAgfVxuICAgIH1cbiAgfVxuXG4gIHJldHVybiBkYXRhO1xufVxuIiwgImV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKGQpIHtcbiAgaWYgKGlzTmFOKHggPSArdGhpcy5feC5jYWxsKG51bGwsIGQpKSB8fCBpc05hTih5ID0gK3RoaXMuX3kuY2FsbChudWxsLCBkKSkpIHJldHVybiB0aGlzOyAvLyBpZ25vcmUgaW52YWxpZCBwb2ludHNcblxuICB2YXIgcGFyZW50LFxuICAgICAgbm9kZSA9IHRoaXMuX3Jvb3QsXG4gICAgICByZXRhaW5lcixcbiAgICAgIHByZXZpb3VzLFxuICAgICAgbmV4dCxcbiAgICAgIHgwID0gdGhpcy5feDAsXG4gICAgICB5MCA9IHRoaXMuX3kwLFxuICAgICAgeDEgPSB0aGlzLl94MSxcbiAgICAgIHkxID0gdGhpcy5feTEsXG4gICAgICB4LFxuICAgICAgeSxcbiAgICAgIHhtLFxuICAgICAgeW0sXG4gICAgICByaWdodCxcbiAgICAgIGJvdHRvbSxcbiAgICAgIGksXG4gICAgICBqO1xuXG4gIC8vIElmIHRoZSB0cmVlIGlzIGVtcHR5LCBpbml0aWFsaXplIHRoZSByb290IGFzIGEgbGVhZi5cbiAgaWYgKCFub2RlKSByZXR1cm4gdGhpcztcblxuICAvLyBGaW5kIHRoZSBsZWFmIG5vZGUgZm9yIHRoZSBwb2ludC5cbiAgLy8gV2hpbGUgZGVzY2VuZGluZywgYWxzbyByZXRhaW4gdGhlIGRlZXBlc3QgcGFyZW50IHdpdGggYSBub24tcmVtb3ZlZCBzaWJsaW5nLlxuICBpZiAobm9kZS5sZW5ndGgpIHdoaWxlICh0cnVlKSB7XG4gICAgaWYgKHJpZ2h0ID0geCA+PSAoeG0gPSAoeDAgKyB4MSkgLyAyKSkgeDAgPSB4bTsgZWxzZSB4MSA9IHhtO1xuICAgIGlmIChib3R0b20gPSB5ID49ICh5bSA9ICh5MCArIHkxKSAvIDIpKSB5MCA9IHltOyBlbHNlIHkxID0geW07XG4gICAgaWYgKCEocGFyZW50ID0gbm9kZSwgbm9kZSA9IG5vZGVbaSA9IGJvdHRvbSA8PCAxIHwgcmlnaHRdKSkgcmV0dXJuIHRoaXM7XG4gICAgaWYgKCFub2RlLmxlbmd0aCkgYnJlYWs7XG4gICAgaWYgKHBhcmVudFsoaSArIDEpICYgM10gfHwgcGFyZW50WyhpICsgMikgJiAzXSB8fCBwYXJlbnRbKGkgKyAzKSAmIDNdKSByZXRhaW5lciA9IHBhcmVudCwgaiA9IGk7XG4gIH1cblxuICAvLyBGaW5kIHRoZSBwb2ludCB0byByZW1vdmUuXG4gIHdoaWxlIChub2RlLmRhdGEgIT09IGQpIGlmICghKHByZXZpb3VzID0gbm9kZSwgbm9kZSA9IG5vZGUubmV4dCkpIHJldHVybiB0aGlzO1xuICBpZiAobmV4dCA9IG5vZGUubmV4dCkgZGVsZXRlIG5vZGUubmV4dDtcblxuICAvLyBJZiB0aGVyZSBhcmUgbXVsdGlwbGUgY29pbmNpZGVudCBwb2ludHMsIHJlbW92ZSBqdXN0IHRoZSBwb2ludC5cbiAgaWYgKHByZXZpb3VzKSByZXR1cm4gKG5leHQgPyBwcmV2aW91cy5uZXh0ID0gbmV4dCA6IGRlbGV0ZSBwcmV2aW91cy5uZXh0KSwgdGhpcztcblxuICAvLyBJZiB0aGlzIGlzIHRoZSByb290IHBvaW50LCByZW1vdmUgaXQuXG4gIGlmICghcGFyZW50KSByZXR1cm4gdGhpcy5fcm9vdCA9IG5leHQsIHRoaXM7XG5cbiAgLy8gUmVtb3ZlIHRoaXMgbGVhZi5cbiAgbmV4dCA/IHBhcmVudFtpXSA9IG5leHQgOiBkZWxldGUgcGFyZW50W2ldO1xuXG4gIC8vIElmIHRoZSBwYXJlbnQgbm93IGNvbnRhaW5zIGV4YWN0bHkgb25lIGxlYWYsIGNvbGxhcHNlIHN1cGVyZmx1b3VzIHBhcmVudHMuXG4gIGlmICgobm9kZSA9IHBhcmVudFswXSB8fCBwYXJlbnRbMV0gfHwgcGFyZW50WzJdIHx8IHBhcmVudFszXSlcbiAgICAgICYmIG5vZGUgPT09IChwYXJlbnRbM10gfHwgcGFyZW50WzJdIHx8IHBhcmVudFsxXSB8fCBwYXJlbnRbMF0pXG4gICAgICAmJiAhbm9kZS5sZW5ndGgpIHtcbiAgICBpZiAocmV0YWluZXIpIHJldGFpbmVyW2pdID0gbm9kZTtcbiAgICBlbHNlIHRoaXMuX3Jvb3QgPSBub2RlO1xuICB9XG5cbiAgcmV0dXJuIHRoaXM7XG59XG5cbmV4cG9ydCBmdW5jdGlvbiByZW1vdmVBbGwoZGF0YSkge1xuICBmb3IgKHZhciBpID0gMCwgbiA9IGRhdGEubGVuZ3RoOyBpIDwgbjsgKytpKSB0aGlzLnJlbW92ZShkYXRhW2ldKTtcbiAgcmV0dXJuIHRoaXM7XG59XG4iLCAiZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24oKSB7XG4gIHJldHVybiB0aGlzLl9yb290O1xufVxuIiwgImV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKCkge1xuICB2YXIgc2l6ZSA9IDA7XG4gIHRoaXMudmlzaXQoZnVuY3Rpb24obm9kZSkge1xuICAgIGlmICghbm9kZS5sZW5ndGgpIGRvICsrc2l6ZTsgd2hpbGUgKG5vZGUgPSBub2RlLm5leHQpXG4gIH0pO1xuICByZXR1cm4gc2l6ZTtcbn1cbiIsICJpbXBvcnQgUXVhZCBmcm9tIFwiLi9xdWFkLmpzXCI7XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKGNhbGxiYWNrKSB7XG4gIHZhciBxdWFkcyA9IFtdLCBxLCBub2RlID0gdGhpcy5fcm9vdCwgY2hpbGQsIHgwLCB5MCwgeDEsIHkxO1xuICBpZiAobm9kZSkgcXVhZHMucHVzaChuZXcgUXVhZChub2RlLCB0aGlzLl94MCwgdGhpcy5feTAsIHRoaXMuX3gxLCB0aGlzLl95MSkpO1xuICB3aGlsZSAocSA9IHF1YWRzLnBvcCgpKSB7XG4gICAgaWYgKCFjYWxsYmFjayhub2RlID0gcS5ub2RlLCB4MCA9IHEueDAsIHkwID0gcS55MCwgeDEgPSBxLngxLCB5MSA9IHEueTEpICYmIG5vZGUubGVuZ3RoKSB7XG4gICAgICB2YXIgeG0gPSAoeDAgKyB4MSkgLyAyLCB5bSA9ICh5MCArIHkxKSAvIDI7XG4gICAgICBpZiAoY2hpbGQgPSBub2RlWzNdKSBxdWFkcy5wdXNoKG5ldyBRdWFkKGNoaWxkLCB4bSwgeW0sIHgxLCB5MSkpO1xuICAgICAgaWYgKGNoaWxkID0gbm9kZVsyXSkgcXVhZHMucHVzaChuZXcgUXVhZChjaGlsZCwgeDAsIHltLCB4bSwgeTEpKTtcbiAgICAgIGlmIChjaGlsZCA9IG5vZGVbMV0pIHF1YWRzLnB1c2gobmV3IFF1YWQoY2hpbGQsIHhtLCB5MCwgeDEsIHltKSk7XG4gICAgICBpZiAoY2hpbGQgPSBub2RlWzBdKSBxdWFkcy5wdXNoKG5ldyBRdWFkKGNoaWxkLCB4MCwgeTAsIHhtLCB5bSkpO1xuICAgIH1cbiAgfVxuICByZXR1cm4gdGhpcztcbn1cbiIsICJpbXBvcnQgUXVhZCBmcm9tIFwiLi9xdWFkLmpzXCI7XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKGNhbGxiYWNrKSB7XG4gIHZhciBxdWFkcyA9IFtdLCBuZXh0ID0gW10sIHE7XG4gIGlmICh0aGlzLl9yb290KSBxdWFkcy5wdXNoKG5ldyBRdWFkKHRoaXMuX3Jvb3QsIHRoaXMuX3gwLCB0aGlzLl95MCwgdGhpcy5feDEsIHRoaXMuX3kxKSk7XG4gIHdoaWxlIChxID0gcXVhZHMucG9wKCkpIHtcbiAgICB2YXIgbm9kZSA9IHEubm9kZTtcbiAgICBpZiAobm9kZS5sZW5ndGgpIHtcbiAgICAgIHZhciBjaGlsZCwgeDAgPSBxLngwLCB5MCA9IHEueTAsIHgxID0gcS54MSwgeTEgPSBxLnkxLCB4bSA9ICh4MCArIHgxKSAvIDIsIHltID0gKHkwICsgeTEpIC8gMjtcbiAgICAgIGlmIChjaGlsZCA9IG5vZGVbMF0pIHF1YWRzLnB1c2gobmV3IFF1YWQoY2hpbGQsIHgwLCB5MCwgeG0sIHltKSk7XG4gICAgICBpZiAoY2hpbGQgPSBub2RlWzFdKSBxdWFkcy5wdXNoKG5ldyBRdWFkKGNoaWxkLCB4bSwgeTAsIHgxLCB5bSkpO1xuICAgICAgaWYgKGNoaWxkID0gbm9kZVsyXSkgcXVhZHMucHVzaChuZXcgUXVhZChjaGlsZCwgeDAsIHltLCB4bSwgeTEpKTtcbiAgICAgIGlmIChjaGlsZCA9IG5vZGVbM10pIHF1YWRzLnB1c2gobmV3IFF1YWQoY2hpbGQsIHhtLCB5bSwgeDEsIHkxKSk7XG4gICAgfVxuICAgIG5leHQucHVzaChxKTtcbiAgfVxuICB3aGlsZSAocSA9IG5leHQucG9wKCkpIHtcbiAgICBjYWxsYmFjayhxLm5vZGUsIHEueDAsIHEueTAsIHEueDEsIHEueTEpO1xuICB9XG4gIHJldHVybiB0aGlzO1xufVxuIiwgImV4cG9ydCBmdW5jdGlvbiBkZWZhdWx0WChkKSB7XG4gIHJldHVybiBkWzBdO1xufVxuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbihfKSB7XG4gIHJldHVybiBhcmd1bWVudHMubGVuZ3RoID8gKHRoaXMuX3ggPSBfLCB0aGlzKSA6IHRoaXMuX3g7XG59XG4iLCAiZXhwb3J0IGZ1bmN0aW9uIGRlZmF1bHRZKGQpIHtcbiAgcmV0dXJuIGRbMV07XG59XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKF8pIHtcbiAgcmV0dXJuIGFyZ3VtZW50cy5sZW5ndGggPyAodGhpcy5feSA9IF8sIHRoaXMpIDogdGhpcy5feTtcbn1cbiIsICJpbXBvcnQgdHJlZV9hZGQsIHthZGRBbGwgYXMgdHJlZV9hZGRBbGx9IGZyb20gXCIuL2FkZC5qc1wiO1xuaW1wb3J0IHRyZWVfY292ZXIgZnJvbSBcIi4vY292ZXIuanNcIjtcbmltcG9ydCB0cmVlX2RhdGEgZnJvbSBcIi4vZGF0YS5qc1wiO1xuaW1wb3J0IHRyZWVfZXh0ZW50IGZyb20gXCIuL2V4dGVudC5qc1wiO1xuaW1wb3J0IHRyZWVfZmluZCBmcm9tIFwiLi9maW5kLmpzXCI7XG5pbXBvcnQgdHJlZV9yZW1vdmUsIHtyZW1vdmVBbGwgYXMgdHJlZV9yZW1vdmVBbGx9IGZyb20gXCIuL3JlbW92ZS5qc1wiO1xuaW1wb3J0IHRyZWVfcm9vdCBmcm9tIFwiLi9yb290LmpzXCI7XG5pbXBvcnQgdHJlZV9zaXplIGZyb20gXCIuL3NpemUuanNcIjtcbmltcG9ydCB0cmVlX3Zpc2l0IGZyb20gXCIuL3Zpc2l0LmpzXCI7XG5pbXBvcnQgdHJlZV92aXNpdEFmdGVyIGZyb20gXCIuL3Zpc2l0QWZ0ZXIuanNcIjtcbmltcG9ydCB0cmVlX3gsIHtkZWZhdWx0WH0gZnJvbSBcIi4veC5qc1wiO1xuaW1wb3J0IHRyZWVfeSwge2RlZmF1bHRZfSBmcm9tIFwiLi95LmpzXCI7XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uIHF1YWR0cmVlKG5vZGVzLCB4LCB5KSB7XG4gIHZhciB0cmVlID0gbmV3IFF1YWR0cmVlKHggPT0gbnVsbCA/IGRlZmF1bHRYIDogeCwgeSA9PSBudWxsID8gZGVmYXVsdFkgOiB5LCBOYU4sIE5hTiwgTmFOLCBOYU4pO1xuICByZXR1cm4gbm9kZXMgPT0gbnVsbCA/IHRyZWUgOiB0cmVlLmFkZEFsbChub2Rlcyk7XG59XG5cbmZ1bmN0aW9uIFF1YWR0cmVlKHgsIHksIHgwLCB5MCwgeDEsIHkxKSB7XG4gIHRoaXMuX3ggPSB4O1xuICB0aGlzLl95ID0geTtcbiAgdGhpcy5feDAgPSB4MDtcbiAgdGhpcy5feTAgPSB5MDtcbiAgdGhpcy5feDEgPSB4MTtcbiAgdGhpcy5feTEgPSB5MTtcbiAgdGhpcy5fcm9vdCA9IHVuZGVmaW5lZDtcbn1cblxuZnVuY3Rpb24gbGVhZl9jb3B5KGxlYWYpIHtcbiAgdmFyIGNvcHkgPSB7ZGF0YTogbGVhZi5kYXRhfSwgbmV4dCA9IGNvcHk7XG4gIHdoaWxlIChsZWFmID0gbGVhZi5uZXh0KSBuZXh0ID0gbmV4dC5uZXh0ID0ge2RhdGE6IGxlYWYuZGF0YX07XG4gIHJldHVybiBjb3B5O1xufVxuXG52YXIgdHJlZVByb3RvID0gcXVhZHRyZWUucHJvdG90eXBlID0gUXVhZHRyZWUucHJvdG90eXBlO1xuXG50cmVlUHJvdG8uY29weSA9IGZ1bmN0aW9uKCkge1xuICB2YXIgY29weSA9IG5ldyBRdWFkdHJlZSh0aGlzLl94LCB0aGlzLl95LCB0aGlzLl94MCwgdGhpcy5feTAsIHRoaXMuX3gxLCB0aGlzLl95MSksXG4gICAgICBub2RlID0gdGhpcy5fcm9vdCxcbiAgICAgIG5vZGVzLFxuICAgICAgY2hpbGQ7XG5cbiAgaWYgKCFub2RlKSByZXR1cm4gY29weTtcblxuICBpZiAoIW5vZGUubGVuZ3RoKSByZXR1cm4gY29weS5fcm9vdCA9IGxlYWZfY29weShub2RlKSwgY29weTtcblxuICBub2RlcyA9IFt7c291cmNlOiBub2RlLCB0YXJnZXQ6IGNvcHkuX3Jvb3QgPSBuZXcgQXJyYXkoNCl9XTtcbiAgd2hpbGUgKG5vZGUgPSBub2Rlcy5wb3AoKSkge1xuICAgIGZvciAodmFyIGkgPSAwOyBpIDwgNDsgKytpKSB7XG4gICAgICBpZiAoY2hpbGQgPSBub2RlLnNvdXJjZVtpXSkge1xuICAgICAgICBpZiAoY2hpbGQubGVuZ3RoKSBub2Rlcy5wdXNoKHtzb3VyY2U6IGNoaWxkLCB0YXJnZXQ6IG5vZGUudGFyZ2V0W2ldID0gbmV3IEFycmF5KDQpfSk7XG4gICAgICAgIGVsc2Ugbm9kZS50YXJnZXRbaV0gPSBsZWFmX2NvcHkoY2hpbGQpO1xuICAgICAgfVxuICAgIH1cbiAgfVxuXG4gIHJldHVybiBjb3B5O1xufTtcblxudHJlZVByb3RvLmFkZCA9IHRyZWVfYWRkO1xudHJlZVByb3RvLmFkZEFsbCA9IHRyZWVfYWRkQWxsO1xudHJlZVByb3RvLmNvdmVyID0gdHJlZV9jb3ZlcjtcbnRyZWVQcm90by5kYXRhID0gdHJlZV9kYXRhO1xudHJlZVByb3RvLmV4dGVudCA9IHRyZWVfZXh0ZW50O1xudHJlZVByb3RvLmZpbmQgPSB0cmVlX2ZpbmQ7XG50cmVlUHJvdG8ucmVtb3ZlID0gdHJlZV9yZW1vdmU7XG50cmVlUHJvdG8ucmVtb3ZlQWxsID0gdHJlZV9yZW1vdmVBbGw7XG50cmVlUHJvdG8ucm9vdCA9IHRyZWVfcm9vdDtcbnRyZWVQcm90by5zaXplID0gdHJlZV9zaXplO1xudHJlZVByb3RvLnZpc2l0ID0gdHJlZV92aXNpdDtcbnRyZWVQcm90by52aXNpdEFmdGVyID0gdHJlZV92aXNpdEFmdGVyO1xudHJlZVByb3RvLnggPSB0cmVlX3g7XG50cmVlUHJvdG8ueSA9IHRyZWVfeTtcbiIsICJleHBvcnQgZGVmYXVsdCBmdW5jdGlvbih4KSB7XG4gIHJldHVybiBmdW5jdGlvbigpIHtcbiAgICByZXR1cm4geDtcbiAgfTtcbn1cbiIsICJleHBvcnQgZGVmYXVsdCBmdW5jdGlvbihyYW5kb20pIHtcbiAgcmV0dXJuIChyYW5kb20oKSAtIDAuNSkgKiAxZS02O1xufVxuIiwgImltcG9ydCB7cXVhZHRyZWV9IGZyb20gXCJkMy1xdWFkdHJlZVwiO1xuaW1wb3J0IGNvbnN0YW50IGZyb20gXCIuL2NvbnN0YW50LmpzXCI7XG5pbXBvcnQgamlnZ2xlIGZyb20gXCIuL2ppZ2dsZS5qc1wiO1xuXG5mdW5jdGlvbiB4KGQpIHtcbiAgcmV0dXJuIGQueCArIGQudng7XG59XG5cbmZ1bmN0aW9uIHkoZCkge1xuICByZXR1cm4gZC55ICsgZC52eTtcbn1cblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24ocmFkaXVzKSB7XG4gIHZhciBub2RlcyxcbiAgICAgIHJhZGlpLFxuICAgICAgcmFuZG9tLFxuICAgICAgc3RyZW5ndGggPSAxLFxuICAgICAgaXRlcmF0aW9ucyA9IDE7XG5cbiAgaWYgKHR5cGVvZiByYWRpdXMgIT09IFwiZnVuY3Rpb25cIikgcmFkaXVzID0gY29uc3RhbnQocmFkaXVzID09IG51bGwgPyAxIDogK3JhZGl1cyk7XG5cbiAgZnVuY3Rpb24gZm9yY2UoKSB7XG4gICAgdmFyIGksIG4gPSBub2Rlcy5sZW5ndGgsXG4gICAgICAgIHRyZWUsXG4gICAgICAgIG5vZGUsXG4gICAgICAgIHhpLFxuICAgICAgICB5aSxcbiAgICAgICAgcmksXG4gICAgICAgIHJpMjtcblxuICAgIGZvciAodmFyIGsgPSAwOyBrIDwgaXRlcmF0aW9uczsgKytrKSB7XG4gICAgICB0cmVlID0gcXVhZHRyZWUobm9kZXMsIHgsIHkpLnZpc2l0QWZ0ZXIocHJlcGFyZSk7XG4gICAgICBmb3IgKGkgPSAwOyBpIDwgbjsgKytpKSB7XG4gICAgICAgIG5vZGUgPSBub2Rlc1tpXTtcbiAgICAgICAgcmkgPSByYWRpaVtub2RlLmluZGV4XSwgcmkyID0gcmkgKiByaTtcbiAgICAgICAgeGkgPSBub2RlLnggKyBub2RlLnZ4O1xuICAgICAgICB5aSA9IG5vZGUueSArIG5vZGUudnk7XG4gICAgICAgIHRyZWUudmlzaXQoYXBwbHkpO1xuICAgICAgfVxuICAgIH1cblxuICAgIGZ1bmN0aW9uIGFwcGx5KHF1YWQsIHgwLCB5MCwgeDEsIHkxKSB7XG4gICAgICB2YXIgZGF0YSA9IHF1YWQuZGF0YSwgcmogPSBxdWFkLnIsIHIgPSByaSArIHJqO1xuICAgICAgaWYgKGRhdGEpIHtcbiAgICAgICAgaWYgKGRhdGEuaW5kZXggPiBub2RlLmluZGV4KSB7XG4gICAgICAgICAgdmFyIHggPSB4aSAtIGRhdGEueCAtIGRhdGEudngsXG4gICAgICAgICAgICAgIHkgPSB5aSAtIGRhdGEueSAtIGRhdGEudnksXG4gICAgICAgICAgICAgIGwgPSB4ICogeCArIHkgKiB5O1xuICAgICAgICAgIGlmIChsIDwgciAqIHIpIHtcbiAgICAgICAgICAgIGlmICh4ID09PSAwKSB4ID0gamlnZ2xlKHJhbmRvbSksIGwgKz0geCAqIHg7XG4gICAgICAgICAgICBpZiAoeSA9PT0gMCkgeSA9IGppZ2dsZShyYW5kb20pLCBsICs9IHkgKiB5O1xuICAgICAgICAgICAgbCA9IChyIC0gKGwgPSBNYXRoLnNxcnQobCkpKSAvIGwgKiBzdHJlbmd0aDtcbiAgICAgICAgICAgIG5vZGUudnggKz0gKHggKj0gbCkgKiAociA9IChyaiAqPSByaikgLyAocmkyICsgcmopKTtcbiAgICAgICAgICAgIG5vZGUudnkgKz0gKHkgKj0gbCkgKiByO1xuICAgICAgICAgICAgZGF0YS52eCAtPSB4ICogKHIgPSAxIC0gcik7XG4gICAgICAgICAgICBkYXRhLnZ5IC09IHkgKiByO1xuICAgICAgICAgIH1cbiAgICAgICAgfVxuICAgICAgICByZXR1cm47XG4gICAgICB9XG4gICAgICByZXR1cm4geDAgPiB4aSArIHIgfHwgeDEgPCB4aSAtIHIgfHwgeTAgPiB5aSArIHIgfHwgeTEgPCB5aSAtIHI7XG4gICAgfVxuICB9XG5cbiAgZnVuY3Rpb24gcHJlcGFyZShxdWFkKSB7XG4gICAgaWYgKHF1YWQuZGF0YSkgcmV0dXJuIHF1YWQuciA9IHJhZGlpW3F1YWQuZGF0YS5pbmRleF07XG4gICAgZm9yICh2YXIgaSA9IHF1YWQuciA9IDA7IGkgPCA0OyArK2kpIHtcbiAgICAgIGlmIChxdWFkW2ldICYmIHF1YWRbaV0uciA+IHF1YWQucikge1xuICAgICAgICBxdWFkLnIgPSBxdWFkW2ldLnI7XG4gICAgICB9XG4gICAgfVxuICB9XG5cbiAgZnVuY3Rpb24gaW5pdGlhbGl6ZSgpIHtcbiAgICBpZiAoIW5vZGVzKSByZXR1cm47XG4gICAgdmFyIGksIG4gPSBub2Rlcy5sZW5ndGgsIG5vZGU7XG4gICAgcmFkaWkgPSBuZXcgQXJyYXkobik7XG4gICAgZm9yIChpID0gMDsgaSA8IG47ICsraSkgbm9kZSA9IG5vZGVzW2ldLCByYWRpaVtub2RlLmluZGV4XSA9ICtyYWRpdXMobm9kZSwgaSwgbm9kZXMpO1xuICB9XG5cbiAgZm9yY2UuaW5pdGlhbGl6ZSA9IGZ1bmN0aW9uKF9ub2RlcywgX3JhbmRvbSkge1xuICAgIG5vZGVzID0gX25vZGVzO1xuICAgIHJhbmRvbSA9IF9yYW5kb207XG4gICAgaW5pdGlhbGl6ZSgpO1xuICB9O1xuXG4gIGZvcmNlLml0ZXJhdGlvbnMgPSBmdW5jdGlvbihfKSB7XG4gICAgcmV0dXJuIGFyZ3VtZW50cy5sZW5ndGggPyAoaXRlcmF0aW9ucyA9ICtfLCBmb3JjZSkgOiBpdGVyYXRpb25zO1xuICB9O1xuXG4gIGZvcmNlLnN0cmVuZ3RoID0gZnVuY3Rpb24oXykge1xuICAgIHJldHVybiBhcmd1bWVudHMubGVuZ3RoID8gKHN0cmVuZ3RoID0gK18sIGZvcmNlKSA6IHN0cmVuZ3RoO1xuICB9O1xuXG4gIGZvcmNlLnJhZGl1cyA9IGZ1bmN0aW9uKF8pIHtcbiAgICByZXR1cm4gYXJndW1lbnRzLmxlbmd0aCA/IChyYWRpdXMgPSB0eXBlb2YgXyA9PT0gXCJmdW5jdGlvblwiID8gXyA6IGNvbnN0YW50KCtfKSwgaW5pdGlhbGl6ZSgpLCBmb3JjZSkgOiByYWRpdXM7XG4gIH07XG5cbiAgcmV0dXJuIGZvcmNlO1xufVxuIiwgImltcG9ydCBjb25zdGFudCBmcm9tIFwiLi9jb25zdGFudC5qc1wiO1xuaW1wb3J0IGppZ2dsZSBmcm9tIFwiLi9qaWdnbGUuanNcIjtcblxuZnVuY3Rpb24gaW5kZXgoZCkge1xuICByZXR1cm4gZC5pbmRleDtcbn1cblxuZnVuY3Rpb24gZmluZChub2RlQnlJZCwgbm9kZUlkKSB7XG4gIHZhciBub2RlID0gbm9kZUJ5SWQuZ2V0KG5vZGVJZCk7XG4gIGlmICghbm9kZSkgdGhyb3cgbmV3IEVycm9yKFwibm9kZSBub3QgZm91bmQ6IFwiICsgbm9kZUlkKTtcbiAgcmV0dXJuIG5vZGU7XG59XG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKGxpbmtzKSB7XG4gIHZhciBpZCA9IGluZGV4LFxuICAgICAgc3RyZW5ndGggPSBkZWZhdWx0U3RyZW5ndGgsXG4gICAgICBzdHJlbmd0aHMsXG4gICAgICBkaXN0YW5jZSA9IGNvbnN0YW50KDMwKSxcbiAgICAgIGRpc3RhbmNlcyxcbiAgICAgIG5vZGVzLFxuICAgICAgY291bnQsXG4gICAgICBiaWFzLFxuICAgICAgcmFuZG9tLFxuICAgICAgaXRlcmF0aW9ucyA9IDE7XG5cbiAgaWYgKGxpbmtzID09IG51bGwpIGxpbmtzID0gW107XG5cbiAgZnVuY3Rpb24gZGVmYXVsdFN0cmVuZ3RoKGxpbmspIHtcbiAgICByZXR1cm4gMSAvIE1hdGgubWluKGNvdW50W2xpbmsuc291cmNlLmluZGV4XSwgY291bnRbbGluay50YXJnZXQuaW5kZXhdKTtcbiAgfVxuXG4gIGZ1bmN0aW9uIGZvcmNlKGFscGhhKSB7XG4gICAgZm9yICh2YXIgayA9IDAsIG4gPSBsaW5rcy5sZW5ndGg7IGsgPCBpdGVyYXRpb25zOyArK2spIHtcbiAgICAgIGZvciAodmFyIGkgPSAwLCBsaW5rLCBzb3VyY2UsIHRhcmdldCwgeCwgeSwgbCwgYjsgaSA8IG47ICsraSkge1xuICAgICAgICBsaW5rID0gbGlua3NbaV0sIHNvdXJjZSA9IGxpbmsuc291cmNlLCB0YXJnZXQgPSBsaW5rLnRhcmdldDtcbiAgICAgICAgeCA9IHRhcmdldC54ICsgdGFyZ2V0LnZ4IC0gc291cmNlLnggLSBzb3VyY2UudnggfHwgamlnZ2xlKHJhbmRvbSk7XG4gICAgICAgIHkgPSB0YXJnZXQueSArIHRhcmdldC52eSAtIHNvdXJjZS55IC0gc291cmNlLnZ5IHx8IGppZ2dsZShyYW5kb20pO1xuICAgICAgICBsID0gTWF0aC5zcXJ0KHggKiB4ICsgeSAqIHkpO1xuICAgICAgICBsID0gKGwgLSBkaXN0YW5jZXNbaV0pIC8gbCAqIGFscGhhICogc3RyZW5ndGhzW2ldO1xuICAgICAgICB4ICo9IGwsIHkgKj0gbDtcbiAgICAgICAgdGFyZ2V0LnZ4IC09IHggKiAoYiA9IGJpYXNbaV0pO1xuICAgICAgICB0YXJnZXQudnkgLT0geSAqIGI7XG4gICAgICAgIHNvdXJjZS52eCArPSB4ICogKGIgPSAxIC0gYik7XG4gICAgICAgIHNvdXJjZS52eSArPSB5ICogYjtcbiAgICAgIH1cbiAgICB9XG4gIH1cblxuICBmdW5jdGlvbiBpbml0aWFsaXplKCkge1xuICAgIGlmICghbm9kZXMpIHJldHVybjtcblxuICAgIHZhciBpLFxuICAgICAgICBuID0gbm9kZXMubGVuZ3RoLFxuICAgICAgICBtID0gbGlua3MubGVuZ3RoLFxuICAgICAgICBub2RlQnlJZCA9IG5ldyBNYXAobm9kZXMubWFwKChkLCBpKSA9PiBbaWQoZCwgaSwgbm9kZXMpLCBkXSkpLFxuICAgICAgICBsaW5rO1xuXG4gICAgZm9yIChpID0gMCwgY291bnQgPSBuZXcgQXJyYXkobik7IGkgPCBtOyArK2kpIHtcbiAgICAgIGxpbmsgPSBsaW5rc1tpXSwgbGluay5pbmRleCA9IGk7XG4gICAgICBpZiAodHlwZW9mIGxpbmsuc291cmNlICE9PSBcIm9iamVjdFwiKSBsaW5rLnNvdXJjZSA9IGZpbmQobm9kZUJ5SWQsIGxpbmsuc291cmNlKTtcbiAgICAgIGlmICh0eXBlb2YgbGluay50YXJnZXQgIT09IFwib2JqZWN0XCIpIGxpbmsudGFyZ2V0ID0gZmluZChub2RlQnlJZCwgbGluay50YXJnZXQpO1xuICAgICAgY291bnRbbGluay5zb3VyY2UuaW5kZXhdID0gKGNvdW50W2xpbmsuc291cmNlLmluZGV4XSB8fCAwKSArIDE7XG4gICAgICBjb3VudFtsaW5rLnRhcmdldC5pbmRleF0gPSAoY291bnRbbGluay50YXJnZXQuaW5kZXhdIHx8IDApICsgMTtcbiAgICB9XG5cbiAgICBmb3IgKGkgPSAwLCBiaWFzID0gbmV3IEFycmF5KG0pOyBpIDwgbTsgKytpKSB7XG4gICAgICBsaW5rID0gbGlua3NbaV0sIGJpYXNbaV0gPSBjb3VudFtsaW5rLnNvdXJjZS5pbmRleF0gLyAoY291bnRbbGluay5zb3VyY2UuaW5kZXhdICsgY291bnRbbGluay50YXJnZXQuaW5kZXhdKTtcbiAgICB9XG5cbiAgICBzdHJlbmd0aHMgPSBuZXcgQXJyYXkobSksIGluaXRpYWxpemVTdHJlbmd0aCgpO1xuICAgIGRpc3RhbmNlcyA9IG5ldyBBcnJheShtKSwgaW5pdGlhbGl6ZURpc3RhbmNlKCk7XG4gIH1cblxuICBmdW5jdGlvbiBpbml0aWFsaXplU3RyZW5ndGgoKSB7XG4gICAgaWYgKCFub2RlcykgcmV0dXJuO1xuXG4gICAgZm9yICh2YXIgaSA9IDAsIG4gPSBsaW5rcy5sZW5ndGg7IGkgPCBuOyArK2kpIHtcbiAgICAgIHN0cmVuZ3Roc1tpXSA9ICtzdHJlbmd0aChsaW5rc1tpXSwgaSwgbGlua3MpO1xuICAgIH1cbiAgfVxuXG4gIGZ1bmN0aW9uIGluaXRpYWxpemVEaXN0YW5jZSgpIHtcbiAgICBpZiAoIW5vZGVzKSByZXR1cm47XG5cbiAgICBmb3IgKHZhciBpID0gMCwgbiA9IGxpbmtzLmxlbmd0aDsgaSA8IG47ICsraSkge1xuICAgICAgZGlzdGFuY2VzW2ldID0gK2Rpc3RhbmNlKGxpbmtzW2ldLCBpLCBsaW5rcyk7XG4gICAgfVxuICB9XG5cbiAgZm9yY2UuaW5pdGlhbGl6ZSA9IGZ1bmN0aW9uKF9ub2RlcywgX3JhbmRvbSkge1xuICAgIG5vZGVzID0gX25vZGVzO1xuICAgIHJhbmRvbSA9IF9yYW5kb207XG4gICAgaW5pdGlhbGl6ZSgpO1xuICB9O1xuXG4gIGZvcmNlLmxpbmtzID0gZnVuY3Rpb24oXykge1xuICAgIHJldHVybiBhcmd1bWVudHMubGVuZ3RoID8gKGxpbmtzID0gXywgaW5pdGlhbGl6ZSgpLCBmb3JjZSkgOiBsaW5rcztcbiAgfTtcblxuICBmb3JjZS5pZCA9IGZ1bmN0aW9uKF8pIHtcbiAgICByZXR1cm4gYXJndW1lbnRzLmxlbmd0aCA/IChpZCA9IF8sIGZvcmNlKSA6IGlkO1xuICB9O1xuXG4gIGZvcmNlLml0ZXJhdGlvbnMgPSBmdW5jdGlvbihfKSB7XG4gICAgcmV0dXJuIGFyZ3VtZW50cy5sZW5ndGggPyAoaXRlcmF0aW9ucyA9ICtfLCBmb3JjZSkgOiBpdGVyYXRpb25zO1xuICB9O1xuXG4gIGZvcmNlLnN0cmVuZ3RoID0gZnVuY3Rpb24oXykge1xuICAgIHJldHVybiBhcmd1bWVudHMubGVuZ3RoID8gKHN0cmVuZ3RoID0gdHlwZW9mIF8gPT09IFwiZnVuY3Rpb25cIiA/IF8gOiBjb25zdGFudCgrXyksIGluaXRpYWxpemVTdHJlbmd0aCgpLCBmb3JjZSkgOiBzdHJlbmd0aDtcbiAgfTtcblxuICBmb3JjZS5kaXN0YW5jZSA9IGZ1bmN0aW9uKF8pIHtcbiAgICByZXR1cm4gYXJndW1lbnRzLmxlbmd0aCA/IChkaXN0YW5jZSA9IHR5cGVvZiBfID09PSBcImZ1bmN0aW9uXCIgPyBfIDogY29uc3RhbnQoK18pLCBpbml0aWFsaXplRGlzdGFuY2UoKSwgZm9yY2UpIDogZGlzdGFuY2U7XG4gIH07XG5cbiAgcmV0dXJuIGZvcmNlO1xufVxuIiwgIi8vIGh0dHBzOi8vZW4ud2lraXBlZGlhLm9yZy93aWtpL0xpbmVhcl9jb25ncnVlbnRpYWxfZ2VuZXJhdG9yI1BhcmFtZXRlcnNfaW5fY29tbW9uX3VzZVxuY29uc3QgYSA9IDE2NjQ1MjU7XG5jb25zdCBjID0gMTAxMzkwNDIyMztcbmNvbnN0IG0gPSA0Mjk0OTY3Mjk2OyAvLyAyXjMyXG5cbmV4cG9ydCBkZWZhdWx0IGZ1bmN0aW9uKCkge1xuICBsZXQgcyA9IDE7XG4gIHJldHVybiAoKSA9PiAocyA9IChhICogcyArIGMpICUgbSkgLyBtO1xufVxuIiwgImltcG9ydCB7ZGlzcGF0Y2h9IGZyb20gXCJkMy1kaXNwYXRjaFwiO1xuaW1wb3J0IHt0aW1lcn0gZnJvbSBcImQzLXRpbWVyXCI7XG5pbXBvcnQgbGNnIGZyb20gXCIuL2xjZy5qc1wiO1xuXG5leHBvcnQgZnVuY3Rpb24geChkKSB7XG4gIHJldHVybiBkLng7XG59XG5cbmV4cG9ydCBmdW5jdGlvbiB5KGQpIHtcbiAgcmV0dXJuIGQueTtcbn1cblxudmFyIGluaXRpYWxSYWRpdXMgPSAxMCxcbiAgICBpbml0aWFsQW5nbGUgPSBNYXRoLlBJICogKDMgLSBNYXRoLnNxcnQoNSkpO1xuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbihub2Rlcykge1xuICB2YXIgc2ltdWxhdGlvbixcbiAgICAgIGFscGhhID0gMSxcbiAgICAgIGFscGhhTWluID0gMC4wMDEsXG4gICAgICBhbHBoYURlY2F5ID0gMSAtIE1hdGgucG93KGFscGhhTWluLCAxIC8gMzAwKSxcbiAgICAgIGFscGhhVGFyZ2V0ID0gMCxcbiAgICAgIHZlbG9jaXR5RGVjYXkgPSAwLjYsXG4gICAgICBmb3JjZXMgPSBuZXcgTWFwKCksXG4gICAgICBzdGVwcGVyID0gdGltZXIoc3RlcCksXG4gICAgICBldmVudCA9IGRpc3BhdGNoKFwidGlja1wiLCBcImVuZFwiKSxcbiAgICAgIHJhbmRvbSA9IGxjZygpO1xuXG4gIGlmIChub2RlcyA9PSBudWxsKSBub2RlcyA9IFtdO1xuXG4gIGZ1bmN0aW9uIHN0ZXAoKSB7XG4gICAgdGljaygpO1xuICAgIGV2ZW50LmNhbGwoXCJ0aWNrXCIsIHNpbXVsYXRpb24pO1xuICAgIGlmIChhbHBoYSA8IGFscGhhTWluKSB7XG4gICAgICBzdGVwcGVyLnN0b3AoKTtcbiAgICAgIGV2ZW50LmNhbGwoXCJlbmRcIiwgc2ltdWxhdGlvbik7XG4gICAgfVxuICB9XG5cbiAgZnVuY3Rpb24gdGljayhpdGVyYXRpb25zKSB7XG4gICAgdmFyIGksIG4gPSBub2Rlcy5sZW5ndGgsIG5vZGU7XG5cbiAgICBpZiAoaXRlcmF0aW9ucyA9PT0gdW5kZWZpbmVkKSBpdGVyYXRpb25zID0gMTtcblxuICAgIGZvciAodmFyIGsgPSAwOyBrIDwgaXRlcmF0aW9uczsgKytrKSB7XG4gICAgICBhbHBoYSArPSAoYWxwaGFUYXJnZXQgLSBhbHBoYSkgKiBhbHBoYURlY2F5O1xuXG4gICAgICBmb3JjZXMuZm9yRWFjaChmdW5jdGlvbihmb3JjZSkge1xuICAgICAgICBmb3JjZShhbHBoYSk7XG4gICAgICB9KTtcblxuICAgICAgZm9yIChpID0gMDsgaSA8IG47ICsraSkge1xuICAgICAgICBub2RlID0gbm9kZXNbaV07XG4gICAgICAgIGlmIChub2RlLmZ4ID09IG51bGwpIG5vZGUueCArPSBub2RlLnZ4ICo9IHZlbG9jaXR5RGVjYXk7XG4gICAgICAgIGVsc2Ugbm9kZS54ID0gbm9kZS5meCwgbm9kZS52eCA9IDA7XG4gICAgICAgIGlmIChub2RlLmZ5ID09IG51bGwpIG5vZGUueSArPSBub2RlLnZ5ICo9IHZlbG9jaXR5RGVjYXk7XG4gICAgICAgIGVsc2Ugbm9kZS55ID0gbm9kZS5meSwgbm9kZS52eSA9IDA7XG4gICAgICB9XG4gICAgfVxuXG4gICAgcmV0dXJuIHNpbXVsYXRpb247XG4gIH1cblxuICBmdW5jdGlvbiBpbml0aWFsaXplTm9kZXMoKSB7XG4gICAgZm9yICh2YXIgaSA9IDAsIG4gPSBub2Rlcy5sZW5ndGgsIG5vZGU7IGkgPCBuOyArK2kpIHtcbiAgICAgIG5vZGUgPSBub2Rlc1tpXSwgbm9kZS5pbmRleCA9IGk7XG4gICAgICBpZiAobm9kZS5meCAhPSBudWxsKSBub2RlLnggPSBub2RlLmZ4O1xuICAgICAgaWYgKG5vZGUuZnkgIT0gbnVsbCkgbm9kZS55ID0gbm9kZS5meTtcbiAgICAgIGlmIChpc05hTihub2RlLngpIHx8IGlzTmFOKG5vZGUueSkpIHtcbiAgICAgICAgdmFyIHJhZGl1cyA9IGluaXRpYWxSYWRpdXMgKiBNYXRoLnNxcnQoMC41ICsgaSksIGFuZ2xlID0gaSAqIGluaXRpYWxBbmdsZTtcbiAgICAgICAgbm9kZS54ID0gcmFkaXVzICogTWF0aC5jb3MoYW5nbGUpO1xuICAgICAgICBub2RlLnkgPSByYWRpdXMgKiBNYXRoLnNpbihhbmdsZSk7XG4gICAgICB9XG4gICAgICBpZiAoaXNOYU4obm9kZS52eCkgfHwgaXNOYU4obm9kZS52eSkpIHtcbiAgICAgICAgbm9kZS52eCA9IG5vZGUudnkgPSAwO1xuICAgICAgfVxuICAgIH1cbiAgfVxuXG4gIGZ1bmN0aW9uIGluaXRpYWxpemVGb3JjZShmb3JjZSkge1xuICAgIGlmIChmb3JjZS5pbml0aWFsaXplKSBmb3JjZS5pbml0aWFsaXplKG5vZGVzLCByYW5kb20pO1xuICAgIHJldHVybiBmb3JjZTtcbiAgfVxuXG4gIGluaXRpYWxpemVOb2RlcygpO1xuXG4gIHJldHVybiBzaW11bGF0aW9uID0ge1xuICAgIHRpY2s6IHRpY2ssXG5cbiAgICByZXN0YXJ0OiBmdW5jdGlvbigpIHtcbiAgICAgIHJldHVybiBzdGVwcGVyLnJlc3RhcnQoc3RlcCksIHNpbXVsYXRpb247XG4gICAgfSxcblxuICAgIHN0b3A6IGZ1bmN0aW9uKCkge1xuICAgICAgcmV0dXJuIHN0ZXBwZXIuc3RvcCgpLCBzaW11bGF0aW9uO1xuICAgIH0sXG5cbiAgICBub2RlczogZnVuY3Rpb24oXykge1xuICAgICAgcmV0dXJuIGFyZ3VtZW50cy5sZW5ndGggPyAobm9kZXMgPSBfLCBpbml0aWFsaXplTm9kZXMoKSwgZm9yY2VzLmZvckVhY2goaW5pdGlhbGl6ZUZvcmNlKSwgc2ltdWxhdGlvbikgOiBub2RlcztcbiAgICB9LFxuXG4gICAgYWxwaGE6IGZ1bmN0aW9uKF8pIHtcbiAgICAgIHJldHVybiBhcmd1bWVudHMubGVuZ3RoID8gKGFscGhhID0gK18sIHNpbXVsYXRpb24pIDogYWxwaGE7XG4gICAgfSxcblxuICAgIGFscGhhTWluOiBmdW5jdGlvbihfKSB7XG4gICAgICByZXR1cm4gYXJndW1lbnRzLmxlbmd0aCA/IChhbHBoYU1pbiA9ICtfLCBzaW11bGF0aW9uKSA6IGFscGhhTWluO1xuICAgIH0sXG5cbiAgICBhbHBoYURlY2F5OiBmdW5jdGlvbihfKSB7XG4gICAgICByZXR1cm4gYXJndW1lbnRzLmxlbmd0aCA/IChhbHBoYURlY2F5ID0gK18sIHNpbXVsYXRpb24pIDogK2FscGhhRGVjYXk7XG4gICAgfSxcblxuICAgIGFscGhhVGFyZ2V0OiBmdW5jdGlvbihfKSB7XG4gICAgICByZXR1cm4gYXJndW1lbnRzLmxlbmd0aCA/IChhbHBoYVRhcmdldCA9ICtfLCBzaW11bGF0aW9uKSA6IGFscGhhVGFyZ2V0O1xuICAgIH0sXG5cbiAgICB2ZWxvY2l0eURlY2F5OiBmdW5jdGlvbihfKSB7XG4gICAgICByZXR1cm4gYXJndW1lbnRzLmxlbmd0aCA/ICh2ZWxvY2l0eURlY2F5ID0gMSAtIF8sIHNpbXVsYXRpb24pIDogMSAtIHZlbG9jaXR5RGVjYXk7XG4gICAgfSxcblxuICAgIHJhbmRvbVNvdXJjZTogZnVuY3Rpb24oXykge1xuICAgICAgcmV0dXJuIGFyZ3VtZW50cy5sZW5ndGggPyAocmFuZG9tID0gXywgZm9yY2VzLmZvckVhY2goaW5pdGlhbGl6ZUZvcmNlKSwgc2ltdWxhdGlvbikgOiByYW5kb207XG4gICAgfSxcblxuICAgIGZvcmNlOiBmdW5jdGlvbihuYW1lLCBfKSB7XG4gICAgICByZXR1cm4gYXJndW1lbnRzLmxlbmd0aCA+IDEgPyAoKF8gPT0gbnVsbCA/IGZvcmNlcy5kZWxldGUobmFtZSkgOiBmb3JjZXMuc2V0KG5hbWUsIGluaXRpYWxpemVGb3JjZShfKSkpLCBzaW11bGF0aW9uKSA6IGZvcmNlcy5nZXQobmFtZSk7XG4gICAgfSxcblxuICAgIGZpbmQ6IGZ1bmN0aW9uKHgsIHksIHJhZGl1cykge1xuICAgICAgdmFyIGkgPSAwLFxuICAgICAgICAgIG4gPSBub2Rlcy5sZW5ndGgsXG4gICAgICAgICAgZHgsXG4gICAgICAgICAgZHksXG4gICAgICAgICAgZDIsXG4gICAgICAgICAgbm9kZSxcbiAgICAgICAgICBjbG9zZXN0O1xuXG4gICAgICBpZiAocmFkaXVzID09IG51bGwpIHJhZGl1cyA9IEluZmluaXR5O1xuICAgICAgZWxzZSByYWRpdXMgKj0gcmFkaXVzO1xuXG4gICAgICBmb3IgKGkgPSAwOyBpIDwgbjsgKytpKSB7XG4gICAgICAgIG5vZGUgPSBub2Rlc1tpXTtcbiAgICAgICAgZHggPSB4IC0gbm9kZS54O1xuICAgICAgICBkeSA9IHkgLSBub2RlLnk7XG4gICAgICAgIGQyID0gZHggKiBkeCArIGR5ICogZHk7XG4gICAgICAgIGlmIChkMiA8IHJhZGl1cykgY2xvc2VzdCA9IG5vZGUsIHJhZGl1cyA9IGQyO1xuICAgICAgfVxuXG4gICAgICByZXR1cm4gY2xvc2VzdDtcbiAgICB9LFxuXG4gICAgb246IGZ1bmN0aW9uKG5hbWUsIF8pIHtcbiAgICAgIHJldHVybiBhcmd1bWVudHMubGVuZ3RoID4gMSA/IChldmVudC5vbihuYW1lLCBfKSwgc2ltdWxhdGlvbikgOiBldmVudC5vbihuYW1lKTtcbiAgICB9XG4gIH07XG59XG4iLCAiaW1wb3J0IHtxdWFkdHJlZX0gZnJvbSBcImQzLXF1YWR0cmVlXCI7XG5pbXBvcnQgY29uc3RhbnQgZnJvbSBcIi4vY29uc3RhbnQuanNcIjtcbmltcG9ydCBqaWdnbGUgZnJvbSBcIi4vamlnZ2xlLmpzXCI7XG5pbXBvcnQge3gsIHl9IGZyb20gXCIuL3NpbXVsYXRpb24uanNcIjtcblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24oKSB7XG4gIHZhciBub2RlcyxcbiAgICAgIG5vZGUsXG4gICAgICByYW5kb20sXG4gICAgICBhbHBoYSxcbiAgICAgIHN0cmVuZ3RoID0gY29uc3RhbnQoLTMwKSxcbiAgICAgIHN0cmVuZ3RocyxcbiAgICAgIGRpc3RhbmNlTWluMiA9IDEsXG4gICAgICBkaXN0YW5jZU1heDIgPSBJbmZpbml0eSxcbiAgICAgIHRoZXRhMiA9IDAuODE7XG5cbiAgZnVuY3Rpb24gZm9yY2UoXykge1xuICAgIHZhciBpLCBuID0gbm9kZXMubGVuZ3RoLCB0cmVlID0gcXVhZHRyZWUobm9kZXMsIHgsIHkpLnZpc2l0QWZ0ZXIoYWNjdW11bGF0ZSk7XG4gICAgZm9yIChhbHBoYSA9IF8sIGkgPSAwOyBpIDwgbjsgKytpKSBub2RlID0gbm9kZXNbaV0sIHRyZWUudmlzaXQoYXBwbHkpO1xuICB9XG5cbiAgZnVuY3Rpb24gaW5pdGlhbGl6ZSgpIHtcbiAgICBpZiAoIW5vZGVzKSByZXR1cm47XG4gICAgdmFyIGksIG4gPSBub2Rlcy5sZW5ndGgsIG5vZGU7XG4gICAgc3RyZW5ndGhzID0gbmV3IEFycmF5KG4pO1xuICAgIGZvciAoaSA9IDA7IGkgPCBuOyArK2kpIG5vZGUgPSBub2Rlc1tpXSwgc3RyZW5ndGhzW25vZGUuaW5kZXhdID0gK3N0cmVuZ3RoKG5vZGUsIGksIG5vZGVzKTtcbiAgfVxuXG4gIGZ1bmN0aW9uIGFjY3VtdWxhdGUocXVhZCkge1xuICAgIHZhciBzdHJlbmd0aCA9IDAsIHEsIGMsIHdlaWdodCA9IDAsIHgsIHksIGk7XG5cbiAgICAvLyBGb3IgaW50ZXJuYWwgbm9kZXMsIGFjY3VtdWxhdGUgZm9yY2VzIGZyb20gY2hpbGQgcXVhZHJhbnRzLlxuICAgIGlmIChxdWFkLmxlbmd0aCkge1xuICAgICAgZm9yICh4ID0geSA9IGkgPSAwOyBpIDwgNDsgKytpKSB7XG4gICAgICAgIGlmICgocSA9IHF1YWRbaV0pICYmIChjID0gTWF0aC5hYnMocS52YWx1ZSkpKSB7XG4gICAgICAgICAgc3RyZW5ndGggKz0gcS52YWx1ZSwgd2VpZ2h0ICs9IGMsIHggKz0gYyAqIHEueCwgeSArPSBjICogcS55O1xuICAgICAgICB9XG4gICAgICB9XG4gICAgICBxdWFkLnggPSB4IC8gd2VpZ2h0O1xuICAgICAgcXVhZC55ID0geSAvIHdlaWdodDtcbiAgICB9XG5cbiAgICAvLyBGb3IgbGVhZiBub2RlcywgYWNjdW11bGF0ZSBmb3JjZXMgZnJvbSBjb2luY2lkZW50IHF1YWRyYW50cy5cbiAgICBlbHNlIHtcbiAgICAgIHEgPSBxdWFkO1xuICAgICAgcS54ID0gcS5kYXRhLng7XG4gICAgICBxLnkgPSBxLmRhdGEueTtcbiAgICAgIGRvIHN0cmVuZ3RoICs9IHN0cmVuZ3Roc1txLmRhdGEuaW5kZXhdO1xuICAgICAgd2hpbGUgKHEgPSBxLm5leHQpO1xuICAgIH1cblxuICAgIHF1YWQudmFsdWUgPSBzdHJlbmd0aDtcbiAgfVxuXG4gIGZ1bmN0aW9uIGFwcGx5KHF1YWQsIHgxLCBfLCB4Mikge1xuICAgIGlmICghcXVhZC52YWx1ZSkgcmV0dXJuIHRydWU7XG5cbiAgICB2YXIgeCA9IHF1YWQueCAtIG5vZGUueCxcbiAgICAgICAgeSA9IHF1YWQueSAtIG5vZGUueSxcbiAgICAgICAgdyA9IHgyIC0geDEsXG4gICAgICAgIGwgPSB4ICogeCArIHkgKiB5O1xuXG4gICAgLy8gQXBwbHkgdGhlIEJhcm5lcy1IdXQgYXBwcm94aW1hdGlvbiBpZiBwb3NzaWJsZS5cbiAgICAvLyBMaW1pdCBmb3JjZXMgZm9yIHZlcnkgY2xvc2Ugbm9kZXM7IHJhbmRvbWl6ZSBkaXJlY3Rpb24gaWYgY29pbmNpZGVudC5cbiAgICBpZiAodyAqIHcgLyB0aGV0YTIgPCBsKSB7XG4gICAgICBpZiAobCA8IGRpc3RhbmNlTWF4Mikge1xuICAgICAgICBpZiAoeCA9PT0gMCkgeCA9IGppZ2dsZShyYW5kb20pLCBsICs9IHggKiB4O1xuICAgICAgICBpZiAoeSA9PT0gMCkgeSA9IGppZ2dsZShyYW5kb20pLCBsICs9IHkgKiB5O1xuICAgICAgICBpZiAobCA8IGRpc3RhbmNlTWluMikgbCA9IE1hdGguc3FydChkaXN0YW5jZU1pbjIgKiBsKTtcbiAgICAgICAgbm9kZS52eCArPSB4ICogcXVhZC52YWx1ZSAqIGFscGhhIC8gbDtcbiAgICAgICAgbm9kZS52eSArPSB5ICogcXVhZC52YWx1ZSAqIGFscGhhIC8gbDtcbiAgICAgIH1cbiAgICAgIHJldHVybiB0cnVlO1xuICAgIH1cblxuICAgIC8vIE90aGVyd2lzZSwgcHJvY2VzcyBwb2ludHMgZGlyZWN0bHkuXG4gICAgZWxzZSBpZiAocXVhZC5sZW5ndGggfHwgbCA+PSBkaXN0YW5jZU1heDIpIHJldHVybjtcblxuICAgIC8vIExpbWl0IGZvcmNlcyBmb3IgdmVyeSBjbG9zZSBub2RlczsgcmFuZG9taXplIGRpcmVjdGlvbiBpZiBjb2luY2lkZW50LlxuICAgIGlmIChxdWFkLmRhdGEgIT09IG5vZGUgfHwgcXVhZC5uZXh0KSB7XG4gICAgICBpZiAoeCA9PT0gMCkgeCA9IGppZ2dsZShyYW5kb20pLCBsICs9IHggKiB4O1xuICAgICAgaWYgKHkgPT09IDApIHkgPSBqaWdnbGUocmFuZG9tKSwgbCArPSB5ICogeTtcbiAgICAgIGlmIChsIDwgZGlzdGFuY2VNaW4yKSBsID0gTWF0aC5zcXJ0KGRpc3RhbmNlTWluMiAqIGwpO1xuICAgIH1cblxuICAgIGRvIGlmIChxdWFkLmRhdGEgIT09IG5vZGUpIHtcbiAgICAgIHcgPSBzdHJlbmd0aHNbcXVhZC5kYXRhLmluZGV4XSAqIGFscGhhIC8gbDtcbiAgICAgIG5vZGUudnggKz0geCAqIHc7XG4gICAgICBub2RlLnZ5ICs9IHkgKiB3O1xuICAgIH0gd2hpbGUgKHF1YWQgPSBxdWFkLm5leHQpO1xuICB9XG5cbiAgZm9yY2UuaW5pdGlhbGl6ZSA9IGZ1bmN0aW9uKF9ub2RlcywgX3JhbmRvbSkge1xuICAgIG5vZGVzID0gX25vZGVzO1xuICAgIHJhbmRvbSA9IF9yYW5kb207XG4gICAgaW5pdGlhbGl6ZSgpO1xuICB9O1xuXG4gIGZvcmNlLnN0cmVuZ3RoID0gZnVuY3Rpb24oXykge1xuICAgIHJldHVybiBhcmd1bWVudHMubGVuZ3RoID8gKHN0cmVuZ3RoID0gdHlwZW9mIF8gPT09IFwiZnVuY3Rpb25cIiA/IF8gOiBjb25zdGFudCgrXyksIGluaXRpYWxpemUoKSwgZm9yY2UpIDogc3RyZW5ndGg7XG4gIH07XG5cbiAgZm9yY2UuZGlzdGFuY2VNaW4gPSBmdW5jdGlvbihfKSB7XG4gICAgcmV0dXJuIGFyZ3VtZW50cy5sZW5ndGggPyAoZGlzdGFuY2VNaW4yID0gXyAqIF8sIGZvcmNlKSA6IE1hdGguc3FydChkaXN0YW5jZU1pbjIpO1xuICB9O1xuXG4gIGZvcmNlLmRpc3RhbmNlTWF4ID0gZnVuY3Rpb24oXykge1xuICAgIHJldHVybiBhcmd1bWVudHMubGVuZ3RoID8gKGRpc3RhbmNlTWF4MiA9IF8gKiBfLCBmb3JjZSkgOiBNYXRoLnNxcnQoZGlzdGFuY2VNYXgyKTtcbiAgfTtcblxuICBmb3JjZS50aGV0YSA9IGZ1bmN0aW9uKF8pIHtcbiAgICByZXR1cm4gYXJndW1lbnRzLmxlbmd0aCA/ICh0aGV0YTIgPSBfICogXywgZm9yY2UpIDogTWF0aC5zcXJ0KHRoZXRhMik7XG4gIH07XG5cbiAgcmV0dXJuIGZvcmNlO1xufVxuIiwgImltcG9ydCBjb25zdGFudCBmcm9tIFwiLi9jb25zdGFudC5qc1wiO1xuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbih4KSB7XG4gIHZhciBzdHJlbmd0aCA9IGNvbnN0YW50KDAuMSksXG4gICAgICBub2RlcyxcbiAgICAgIHN0cmVuZ3RocyxcbiAgICAgIHh6O1xuXG4gIGlmICh0eXBlb2YgeCAhPT0gXCJmdW5jdGlvblwiKSB4ID0gY29uc3RhbnQoeCA9PSBudWxsID8gMCA6ICt4KTtcblxuICBmdW5jdGlvbiBmb3JjZShhbHBoYSkge1xuICAgIGZvciAodmFyIGkgPSAwLCBuID0gbm9kZXMubGVuZ3RoLCBub2RlOyBpIDwgbjsgKytpKSB7XG4gICAgICBub2RlID0gbm9kZXNbaV0sIG5vZGUudnggKz0gKHh6W2ldIC0gbm9kZS54KSAqIHN0cmVuZ3Roc1tpXSAqIGFscGhhO1xuICAgIH1cbiAgfVxuXG4gIGZ1bmN0aW9uIGluaXRpYWxpemUoKSB7XG4gICAgaWYgKCFub2RlcykgcmV0dXJuO1xuICAgIHZhciBpLCBuID0gbm9kZXMubGVuZ3RoO1xuICAgIHN0cmVuZ3RocyA9IG5ldyBBcnJheShuKTtcbiAgICB4eiA9IG5ldyBBcnJheShuKTtcbiAgICBmb3IgKGkgPSAwOyBpIDwgbjsgKytpKSB7XG4gICAgICBzdHJlbmd0aHNbaV0gPSBpc05hTih4eltpXSA9ICt4KG5vZGVzW2ldLCBpLCBub2RlcykpID8gMCA6ICtzdHJlbmd0aChub2Rlc1tpXSwgaSwgbm9kZXMpO1xuICAgIH1cbiAgfVxuXG4gIGZvcmNlLmluaXRpYWxpemUgPSBmdW5jdGlvbihfKSB7XG4gICAgbm9kZXMgPSBfO1xuICAgIGluaXRpYWxpemUoKTtcbiAgfTtcblxuICBmb3JjZS5zdHJlbmd0aCA9IGZ1bmN0aW9uKF8pIHtcbiAgICByZXR1cm4gYXJndW1lbnRzLmxlbmd0aCA/IChzdHJlbmd0aCA9IHR5cGVvZiBfID09PSBcImZ1bmN0aW9uXCIgPyBfIDogY29uc3RhbnQoK18pLCBpbml0aWFsaXplKCksIGZvcmNlKSA6IHN0cmVuZ3RoO1xuICB9O1xuXG4gIGZvcmNlLnggPSBmdW5jdGlvbihfKSB7XG4gICAgcmV0dXJuIGFyZ3VtZW50cy5sZW5ndGggPyAoeCA9IHR5cGVvZiBfID09PSBcImZ1bmN0aW9uXCIgPyBfIDogY29uc3RhbnQoK18pLCBpbml0aWFsaXplKCksIGZvcmNlKSA6IHg7XG4gIH07XG5cbiAgcmV0dXJuIGZvcmNlO1xufVxuIiwgImltcG9ydCBjb25zdGFudCBmcm9tIFwiLi9jb25zdGFudC5qc1wiO1xuXG5leHBvcnQgZGVmYXVsdCBmdW5jdGlvbih5KSB7XG4gIHZhciBzdHJlbmd0aCA9IGNvbnN0YW50KDAuMSksXG4gICAgICBub2RlcyxcbiAgICAgIHN0cmVuZ3RocyxcbiAgICAgIHl6O1xuXG4gIGlmICh0eXBlb2YgeSAhPT0gXCJmdW5jdGlvblwiKSB5ID0gY29uc3RhbnQoeSA9PSBudWxsID8gMCA6ICt5KTtcblxuICBmdW5jdGlvbiBmb3JjZShhbHBoYSkge1xuICAgIGZvciAodmFyIGkgPSAwLCBuID0gbm9kZXMubGVuZ3RoLCBub2RlOyBpIDwgbjsgKytpKSB7XG4gICAgICBub2RlID0gbm9kZXNbaV0sIG5vZGUudnkgKz0gKHl6W2ldIC0gbm9kZS55KSAqIHN0cmVuZ3Roc1tpXSAqIGFscGhhO1xuICAgIH1cbiAgfVxuXG4gIGZ1bmN0aW9uIGluaXRpYWxpemUoKSB7XG4gICAgaWYgKCFub2RlcykgcmV0dXJuO1xuICAgIHZhciBpLCBuID0gbm9kZXMubGVuZ3RoO1xuICAgIHN0cmVuZ3RocyA9IG5ldyBBcnJheShuKTtcbiAgICB5eiA9IG5ldyBBcnJheShuKTtcbiAgICBmb3IgKGkgPSAwOyBpIDwgbjsgKytpKSB7XG4gICAgICBzdHJlbmd0aHNbaV0gPSBpc05hTih5eltpXSA9ICt5KG5vZGVzW2ldLCBpLCBub2RlcykpID8gMCA6ICtzdHJlbmd0aChub2Rlc1tpXSwgaSwgbm9kZXMpO1xuICAgIH1cbiAgfVxuXG4gIGZvcmNlLmluaXRpYWxpemUgPSBmdW5jdGlvbihfKSB7XG4gICAgbm9kZXMgPSBfO1xuICAgIGluaXRpYWxpemUoKTtcbiAgfTtcblxuICBmb3JjZS5zdHJlbmd0aCA9IGZ1bmN0aW9uKF8pIHtcbiAgICByZXR1cm4gYXJndW1lbnRzLmxlbmd0aCA/IChzdHJlbmd0aCA9IHR5cGVvZiBfID09PSBcImZ1bmN0aW9uXCIgPyBfIDogY29uc3RhbnQoK18pLCBpbml0aWFsaXplKCksIGZvcmNlKSA6IHN0cmVuZ3RoO1xuICB9O1xuXG4gIGZvcmNlLnkgPSBmdW5jdGlvbihfKSB7XG4gICAgcmV0dXJuIGFyZ3VtZW50cy5sZW5ndGggPyAoeSA9IHR5cGVvZiBfID09PSBcImZ1bmN0aW9uXCIgPyBfIDogY29uc3RhbnQoK18pLCBpbml0aWFsaXplKCksIGZvcmNlKSA6IHk7XG4gIH07XG5cbiAgcmV0dXJuIGZvcmNlO1xufVxuIiwgImV4cG9ydCBmdW5jdGlvbiBUcmFuc2Zvcm0oaywgeCwgeSkge1xuICB0aGlzLmsgPSBrO1xuICB0aGlzLnggPSB4O1xuICB0aGlzLnkgPSB5O1xufVxuXG5UcmFuc2Zvcm0ucHJvdG90eXBlID0ge1xuICBjb25zdHJ1Y3RvcjogVHJhbnNmb3JtLFxuICBzY2FsZTogZnVuY3Rpb24oaykge1xuICAgIHJldHVybiBrID09PSAxID8gdGhpcyA6IG5ldyBUcmFuc2Zvcm0odGhpcy5rICogaywgdGhpcy54LCB0aGlzLnkpO1xuICB9LFxuICB0cmFuc2xhdGU6IGZ1bmN0aW9uKHgsIHkpIHtcbiAgICByZXR1cm4geCA9PT0gMCAmIHkgPT09IDAgPyB0aGlzIDogbmV3IFRyYW5zZm9ybSh0aGlzLmssIHRoaXMueCArIHRoaXMuayAqIHgsIHRoaXMueSArIHRoaXMuayAqIHkpO1xuICB9LFxuICBhcHBseTogZnVuY3Rpb24ocG9pbnQpIHtcbiAgICByZXR1cm4gW3BvaW50WzBdICogdGhpcy5rICsgdGhpcy54LCBwb2ludFsxXSAqIHRoaXMuayArIHRoaXMueV07XG4gIH0sXG4gIGFwcGx5WDogZnVuY3Rpb24oeCkge1xuICAgIHJldHVybiB4ICogdGhpcy5rICsgdGhpcy54O1xuICB9LFxuICBhcHBseVk6IGZ1bmN0aW9uKHkpIHtcbiAgICByZXR1cm4geSAqIHRoaXMuayArIHRoaXMueTtcbiAgfSxcbiAgaW52ZXJ0OiBmdW5jdGlvbihsb2NhdGlvbikge1xuICAgIHJldHVybiBbKGxvY2F0aW9uWzBdIC0gdGhpcy54KSAvIHRoaXMuaywgKGxvY2F0aW9uWzFdIC0gdGhpcy55KSAvIHRoaXMua107XG4gIH0sXG4gIGludmVydFg6IGZ1bmN0aW9uKHgpIHtcbiAgICByZXR1cm4gKHggLSB0aGlzLngpIC8gdGhpcy5rO1xuICB9LFxuICBpbnZlcnRZOiBmdW5jdGlvbih5KSB7XG4gICAgcmV0dXJuICh5IC0gdGhpcy55KSAvIHRoaXMuaztcbiAgfSxcbiAgcmVzY2FsZVg6IGZ1bmN0aW9uKHgpIHtcbiAgICByZXR1cm4geC5jb3B5KCkuZG9tYWluKHgucmFuZ2UoKS5tYXAodGhpcy5pbnZlcnRYLCB0aGlzKS5tYXAoeC5pbnZlcnQsIHgpKTtcbiAgfSxcbiAgcmVzY2FsZVk6IGZ1bmN0aW9uKHkpIHtcbiAgICByZXR1cm4geS5jb3B5KCkuZG9tYWluKHkucmFuZ2UoKS5tYXAodGhpcy5pbnZlcnRZLCB0aGlzKS5tYXAoeS5pbnZlcnQsIHkpKTtcbiAgfSxcbiAgdG9TdHJpbmc6IGZ1bmN0aW9uKCkge1xuICAgIHJldHVybiBcInRyYW5zbGF0ZShcIiArIHRoaXMueCArIFwiLFwiICsgdGhpcy55ICsgXCIpIHNjYWxlKFwiICsgdGhpcy5rICsgXCIpXCI7XG4gIH1cbn07XG5cbmV4cG9ydCB2YXIgaWRlbnRpdHkgPSBuZXcgVHJhbnNmb3JtKDEsIDAsIDApO1xuXG50cmFuc2Zvcm0ucHJvdG90eXBlID0gVHJhbnNmb3JtLnByb3RvdHlwZTtcblxuZXhwb3J0IGRlZmF1bHQgZnVuY3Rpb24gdHJhbnNmb3JtKG5vZGUpIHtcbiAgd2hpbGUgKCFub2RlLl9fem9vbSkgaWYgKCEobm9kZSA9IG5vZGUucGFyZW50Tm9kZSkpIHJldHVybiBpZGVudGl0eTtcbiAgcmV0dXJuIG5vZGUuX196b29tO1xufVxuIiwgImV4cG9ydCBpbnRlcmZhY2UgUGh5c2ljc0NvbmZpZyB7XG4gIGNoYXJnZVN0cmVuZ3RoOiBudW1iZXI7XG4gIGNoYXJnZURpc3RhbmNlTWF4OiBudW1iZXI7IC8vIE1heGltdW0gZGlzdGFuY2UgZm9yIGNoYXJnZSBmb3JjZSBjYWxjdWxhdGlvbnMgKHBlcmZvcm1hbmNlKVxuICBjaGFyZ2VUaGV0YTogbnVtYmVyOyAvLyBCYXJuZXMtSHV0IGFwcHJveGltYXRpb24gYWNjdXJhY3kgKDAuOSA9IGFjY3VyYXRlLCAxLjUgPSBmYXN0KVxuICBjb2xsaXNpb25QYWRkaW5nOiBudW1iZXI7XG4gIGxpbmtEaXN0YW5jZTogbnVtYmVyO1xuICBsaW5rU3RyZW5ndGg6IG51bWJlcjtcbiAgdmVsb2NpdHlEZWNheTogbnVtYmVyO1xuICBhbHBoYURlY2F5OiBudW1iZXI7XG4gIHhTdHJlbmd0aDogbnVtYmVyOyAvLyBmb3JjZVggc3RyZW5ndGggLSBhdHRyYWN0cyBub2RlcyBob3Jpem9udGFsbHkgdG8gY2VudGVyXG4gIHlTdHJlbmd0aDogbnVtYmVyOyAvLyBmb3JjZVkgc3RyZW5ndGggLSBhdHRyYWN0cyBub2RlcyB2ZXJ0aWNhbGx5IHRvIGNlbnRlclxufVxuXG5leHBvcnQgY29uc3QgREVGQVVMVF9QSFlTSUNTX0NPTkZJRzogUGh5c2ljc0NvbmZpZyA9IHtcbiAgY2hhcmdlU3RyZW5ndGg6IC02MDAsICAgIC8vIEluY3JlYXNlZCByZXB1bHNpb24gZm9yIGxhcmdlciBub2Rlc1xuICBjaGFyZ2VEaXN0YW5jZU1heDogMjAwMCwgLy8gRWZmZWN0aXZlbHkgZ2xvYmFsIGNoYXJnZSBmb3JjZSBmb3IgbW9yZSBuYXR1cmFsIGxheW91dFxuICBjaGFyZ2VUaGV0YTogMC45LCAgICAgICAgLy8gRGVmYXVsdCBCYXJuZXMtSHV0IGFjY3VyYWN5XG4gIGNvbGxpc2lvblBhZGRpbmc6IDM1LCAgICAvLyBJbmNyZWFzZWQgcGFkZGluZyBmb3IgMTAwOjUwOjIwIG5vZGUgc2l6ZXNcbiAgbGlua0Rpc3RhbmNlOiAxNTAsICAgICAgIC8vIExvbmdlciBlZGdlcyB0byBhY2NvbW1vZGF0ZSBsYXJnZXIgbm9kZXNcbiAgbGlua1N0cmVuZ3RoOiAxLFxuICB2ZWxvY2l0eURlY2F5OiAwLjgsICAgICAgLy8gSW5jcmVhc2VkIGZyaWN0aW9uIGZvciBmYXN0ZXIgc2V0dGxpbmdcbiAgYWxwaGFEZWNheTogMC4wMjI4LCAgICAgIC8vIEQzIGRlZmF1bHQgLSBhbGxvd3MgbW9yZSB0aW1lIGZvciBsYXlvdXQgdG8gc2V0dGxlIHByb3Blcmx5XG4gIHhTdHJlbmd0aDogMC4wMDIsICAgICAgICAvLyBHZW50bGUgZ3Jhdml0eSB0byBwcmV2ZW50IGRpc2Nvbm5lY3RlZCBzdWJncmFwaHMgZnJvbSBkcmlmdGluZ1xuICB5U3RyZW5ndGg6IDAuMDAyLCAgICAgICAgLy8gR2VudGxlIGdyYXZpdHkgdG8gcHJldmVudCBkaXNjb25uZWN0ZWQgc3ViZ3JhcGhzIGZyb20gZHJpZnRpbmdcbn07XG5cbmV4cG9ydCBjb25zdCBTUEFDSU9VU19QSFlTSUNTX0NPTkZJRzogUGh5c2ljc0NvbmZpZyA9IHtcbiAgY2hhcmdlU3RyZW5ndGg6IC04MDAsICAgIC8vIFN0cm9uZyByZXB1bHNpb24gZm9yIHNwYWNpb3VzIGxheW91dCB3aXRoIGxhcmdlIG5vZGVzXG4gIGNoYXJnZURpc3RhbmNlTWF4OiAyMDAwLCAvLyBFZmZlY3RpdmVseSBnbG9iYWwgY2hhcmdlIGZvcmNlIGZvciBtb3JlIG5hdHVyYWwgbGF5b3V0XG4gIGNoYXJnZVRoZXRhOiAwLjksXG4gIGNvbGxpc2lvblBhZGRpbmc6IDQ1LCAgICAvLyBFeHRyYSBwYWRkaW5nIGZvciBzcGFjaW91cyBmZWVsIHdpdGggMTAwOjUwOjIwIHNpemVzXG4gIGxpbmtEaXN0YW5jZTogMTgwLCAgICAgICAvLyBMb25nIGxpbmtzIGZvciBzcGFjaW91cyBsYXlvdXRcbiAgbGlua1N0cmVuZ3RoOiAwLjgsXG4gIHZlbG9jaXR5RGVjYXk6IDAuOCwgICAgICAvLyBJbmNyZWFzZWQgZnJpY3Rpb24gZm9yIGZhc3RlciBzZXR0bGluZ1xuICBhbHBoYURlY2F5OiAwLjAyMjgsXG4gIHhTdHJlbmd0aDogMC4wMDIsICAgICAgICAvLyBHZW50bGUgZ3Jhdml0eSB0byBwcmV2ZW50IGRpc2Nvbm5lY3RlZCBzdWJncmFwaHMgZnJvbSBkcmlmdGluZ1xuICB5U3RyZW5ndGg6IDAuMDAyLCAgICAgICAgLy8gR2VudGxlIGdyYXZpdHkgdG8gcHJldmVudCBkaXNjb25uZWN0ZWQgc3ViZ3JhcGhzIGZyb20gZHJpZnRpbmdcbn07XG5cbi8vIFBlcmZvcm1hbmNlLW9wdGltaXplZCBjb25maWcgZm9yIHZlcnkgbGFyZ2UgZ3JhcGhzICgxMDAwKyBub2RlcylcbmV4cG9ydCBjb25zdCBQRVJGT1JNQU5DRV9QSFlTSUNTX0NPTkZJRzogUGh5c2ljc0NvbmZpZyA9IHtcbiAgY2hhcmdlU3RyZW5ndGg6IC00MDAsICAgIC8vIE1vZGVyYXRlIHJlcHVsc2lvbiBmb3IgcGVyZm9ybWFuY2VcbiAgY2hhcmdlRGlzdGFuY2VNYXg6IDIwMDAsIC8vIEVmZmVjdGl2ZWx5IGdsb2JhbCBjaGFyZ2UgZm9yY2UsIHNpbXBsaWZpZWQgY29uZmlndXJhdGlvblxuICBjaGFyZ2VUaGV0YTogMS4yLCAgICAgICAgLy8gTGVzcyBhY2N1cmF0ZSBidXQgbXVjaCBmYXN0ZXJcbiAgY29sbGlzaW9uUGFkZGluZzogMjUsICAgIC8vIE1pbmltdW0gcGFkZGluZyBmb3IgMTAwOjUwOjIwIHNpemVzXG4gIGxpbmtEaXN0YW5jZTogMTIwLCAgICAgICAvLyBTaG9ydGVyIGxpbmtzIGZvciBwZXJmb3JtYW5jZVxuICBsaW5rU3RyZW5ndGg6IDEsXG4gIHZlbG9jaXR5RGVjYXk6IDAuOCwgICAgICAvLyBIaWdoIGRhbXBpbmcgZm9yIHF1aWNrIHNldHRsaW5nXG4gIGFscGhhRGVjYXk6IDAuMSwgICAgICAgICAvLyBGYXN0IGNvb2xkb3duXG4gIHhTdHJlbmd0aDogMC4wMDIsICAgICAgICAvLyBHZW50bGUgZ3Jhdml0eSB0byBwcmV2ZW50IGRpc2Nvbm5lY3RlZCBzdWJncmFwaHMgZnJvbSBkcmlmdGluZ1xuICB5U3RyZW5ndGg6IDAuMDAyLCAgICAgICAgLy8gR2VudGxlIGdyYXZpdHkgdG8gcHJldmVudCBkaXNjb25uZWN0ZWQgc3ViZ3JhcGhzIGZyb20gZHJpZnRpbmdcbn07IiwgIi8qKlxuICogWm9vbSB0aHJlc2hvbGQgZm9yIERvbWFpbi10by1Eb21haW4gZWRnZXMgdG8gcmVtYWluIHZpc2libGVcbiAqIFxuICogU2V0IG11Y2ggbG93ZXIgdGhhbiBvdGhlciBlZGdlcyBzbyBkb21haW4gcmVsYXRpb25zaGlwcyByZW1haW4gdmlzaWJsZVxuICogd2hlbiB6b29tZWQgb3V0IHRvIHNlZSBhcmNoaXRlY3R1cmFsIG92ZXJ2aWV3LlxuICovXG5leHBvcnQgY29uc3QgRE9NQUlOX0VER0VfTE9EX1RIUkVTSE9MRCA9IDAuMztcblxuLyoqXG4gKiBab29tIHRocmVzaG9sZCBmb3Igc3dpdGNoaW5nIGJldHdlZW4gRG9tYWluIHBpbGxzIGFuZCBMb0QgbGFiZWxzXG4gKiBcbiAqIFNldCBsb3dlciB0aGFuIFN1YmRvbWFpbnMgdG8ga2VlcCBoaWdoLWxldmVsIGNvbnRleHQgdmlzaWJsZSBsb25nZXIuXG4gKiBDYWxjdWxhdGlvbiBmb3Igc2VhbWxlc3MgaGFuZG9mZjpcbiAqIC0gRG9tYWluIHBpbGxzIHVzZSAxNnB4IGZvbnRcbiAqIC0gQXQgdGhyZXNob2xkIDAuNTU6IExvRCBsYWJlbHMgdXNlIGZvbnRTaXplID0gOC44LzAuNTUgPSAxNnB4XG4gKiAtIFRoaXMgY3JlYXRlcyBhIHBlcmZlY3QgbWF0Y2ggYXQgdGhlIHRyYW5zaXRpb24gcG9pbnRcbiAqL1xuZXhwb3J0IGNvbnN0IERPTUFJTl9MT0RfVEhSRVNIT0xEID0gMC40NTtcblxuLyoqXG4gKiBab29tIHRocmVzaG9sZCBmb3Igc3dpdGNoaW5nIGJldHdlZW4gU3ViZG9tYWluIHBpbGxzIGFuZCBmYWRpbmcgb3V0XG4gKiBcbiAqIExvd2VyZWQgdG8ga2VlcCBTdWJkb21haW4gY29udGV4dCB2aXNpYmxlIGxvbmdlciBvbiBzbWFsbGVyIGRpc3BsYXlzLlxuICogU3ViZG9tYWlucyBmYWRlIG91dCBqdXN0IGJlbG93IHRoaXMgdGhyZXNob2xkLCBiYWxhbmNpbmcgY2xhcml0eSBhbmQgY2x1dHRlci5cbiAqL1xuZXhwb3J0IGNvbnN0IFNVQkRPTUFJTl9MT0RfVEhSRVNIT0xEID0gMC4zO1xuXG4vKipcbiAqIEJhc2UgdmlzdWFsIHNpemUgKGluIHBpeGVscykgZm9yIG5vbi1waWxsIG5vZGVzIChmdW5jdGlvbnMsIGZpbGVzLCBldGMuKVxuICpcbiAqIENlbnRyYWxpemVzIHNpemluZyBzbyBsYXlvdXQsIHJlbmRlcmluZywgYW5kIHBoeXNpY3Mgc3RheSBpbiBzeW5jLlxuICovXG5leHBvcnQgY29uc3QgUkVHVUxBUl9OT0RFX0JBU0VfU0laRSA9IDQwO1xuXG4vKipcbiAqIENvbnZlbmllbmNlIGhlbHBlcnMgZGVyaXZlZCBmcm9tIHRoZSBiYXNlIHNpemUuXG4gKi9cbmV4cG9ydCBjb25zdCBSRUdVTEFSX05PREVfSEFMRl9TSVpFID0gUkVHVUxBUl9OT0RFX0JBU0VfU0laRSAvIDI7XG5leHBvcnQgY29uc3QgUkVHVUxBUl9OT0RFX0NPUk5FUl9SQURJVVMgPSBNYXRoLnJvdW5kKFJFR1VMQVJfTk9ERV9CQVNFX1NJWkUgKiAwLjIyKTtcbmV4cG9ydCBjb25zdCBSRUdVTEFSX05PREVfSUNPTl9TQ0FMRSA9IFJFR1VMQVJfTk9ERV9CQVNFX1NJWkUgLyA2MDsgLy8gRmlsZS9kaXIgU1ZHcyBhcmUgNjBweCBhcnRib2FyZHNcblxuZXhwb3J0IGNvbnN0IFJFR1VMQVJfTk9ERV9QSU5fT1VURVJfU1RST0tFID0gJyNCNThENjknO1xuZXhwb3J0IGNvbnN0IFJFR1VMQVJfTk9ERV9QSU5fSU5ORVJfU1RST0tFID0gJyMxQTExMDknO1xuZXhwb3J0IGNvbnN0IFJFR1VMQVJfTk9ERV9QSU5fT1VURVJfU1RST0tFX1dJRFRIID0gTWF0aC5tYXgoMSwgTWF0aC5yb3VuZChSRUdVTEFSX05PREVfQkFTRV9TSVpFICogMC4wNikpO1xuZXhwb3J0IGNvbnN0IFJFR1VMQVJfTk9ERV9QSU5fSU5ORVJfU1RST0tFX1dJRFRIID0gTWF0aC5tYXgoMSwgTWF0aC5yb3VuZChSRUdVTEFSX05PREVfQkFTRV9TSVpFICogMC4wNykpO1xuZXhwb3J0IGNvbnN0IERJUkVDVE9SWV9QSU5fSUNPTl9PRkZTRVRfWSA9IE1hdGgucm91bmQoUkVHVUxBUl9OT0RFX0JBU0VfU0laRSAqIDAuMSk7XG5leHBvcnQgY29uc3QgRklMRV9QSU5fSUNPTl9PRkZTRVRfWSA9IE1hdGgucm91bmQoUkVHVUxBUl9OT0RFX0JBU0VfU0laRSAqIDAuMDYpO1xuXG5leHBvcnQgY29uc3QgUkVHVUxBUl9OT0RFX0xBQkVMX01JTl9TQ1JFRU5fU0laRSA9IE1hdGgubWF4KDksIE1hdGgucm91bmQoUkVHVUxBUl9OT0RFX0JBU0VfU0laRSAqIDAuMjUpKTtcbmV4cG9ydCBjb25zdCBSRUdVTEFSX05PREVfTEFCRUxfVEFSR0VUX1NDUkVFTl9TSVpFID0gTWF0aC5yb3VuZChSRUdVTEFSX05PREVfQkFTRV9TSVpFICogMC4zKTtcbmV4cG9ydCBjb25zdCBSRUdVTEFSX05PREVfTEFCRUxfTUFYX1NDUkVFTl9TSVpFID0gTWF0aC5yb3VuZChSRUdVTEFSX05PREVfQkFTRV9TSVpFICogMC4zMik7XG5leHBvcnQgY29uc3QgUkVHVUxBUl9MQUJFTF9TVFlMRSA9IHtcbiAgcGFkZGluZ1g6IDYsXG4gIHBhZGRpbmdZOiA0LFxuICB2ZXJ0aWNhbEFuY2hvck9mZnNldDogNCxcbn07XG5cbmV4cG9ydCBjb25zdCBSRUdVTEFSX05PREVfUkVQVUxTSU9OX01VTFRJUExJRVIgPSA3LjU7IC8vIFJldHVybiB0byB0aGUgcHJlLWV4cGFuc2lvbiBiYXNlbGluZVxuZXhwb3J0IGNvbnN0IFNVQkRPTUFJTl9OT0RFX1JFUFVMU0lPTl9NVUxUSVBMSUVSID0gMjI7XG5leHBvcnQgY29uc3QgRE9NQUlOX05PREVfUkVQVUxTSU9OX01VTFRJUExJRVIgPSA0MjtcblxuZXhwb3J0IGNvbnN0IE5PREVfREVHUkVFX1JFUFVMU0lPTl9MT0dfU0NBTEUgPSAwLjQ7XG5leHBvcnQgY29uc3QgUkVHVUxBUl9OT0RFX01BWF9ERUdSRUVfQk9PU1QgPSAxLjQ7XG5leHBvcnQgY29uc3QgU1VCRE9NQUlOX05PREVfTUFYX0RFR1JFRV9CT09TVCA9IDEuNjU7XG5leHBvcnQgY29uc3QgRE9NQUlOX05PREVfTUFYX0RFR1JFRV9CT09TVCA9IDEuOTtcbiIsICJpbXBvcnQgeyBSRUdVTEFSX05PREVfQkFTRV9TSVpFIH0gZnJvbSAnLi4vY29uc3RhbnRzJztcblxuZXhwb3J0IGNvbnN0IExJTktfUEFERElORyA9IHtcbiAgZG9tYWluRG9tYWluOiA2MDAsXG4gIGRvbWFpbk90aGVyOiAzNDUsXG4gIHN1YmRvbWFpbk90aGVyOiAyNTMsXG4gIGRlZmF1bHQ6IDE4NlxufTtcblxuZXhwb3J0IGNvbnN0IEJBU0VfQ0hBUkdFID0ge1xuICBEb21haW46IC00MDAwLFxuICBTdWJkb21haW46IC0yMjQwLFxuICBkZWZhdWx0OiAtMTI1MFxufTtcblxuZXhwb3J0IGNvbnN0IEJBU0VfTElOS19TVFJFTkdUSCA9IHtcbiAgZG9tYWluRG9tYWluOiAwLjA0LFxuICBkb21haW5PdGhlcjogMC4yMSxcbiAgZGVmYXVsdDogMC40MlxufTtcblxuZXhwb3J0IGNvbnN0IENPTExJU0lPTl9QQURESU5HID0ge1xuICBEb21haW46IDYwLFxuICBTdWJkb21haW46IDUwLFxuICBkZWZhdWx0OiA0MlxufSBhcyBjb25zdDtcbmV4cG9ydCBjb25zdCBCQVNFX0NFTlRFUl9TVFJFTkdUSCA9IDAuMDAwNTtcblxuZXhwb3J0IGZ1bmN0aW9uIGdldERlbnNpdHlGYWN0b3Iobm9kZUNvdW50OiBudW1iZXIpOiBudW1iZXIge1xuICByZXR1cm4gbm9kZUNvdW50ID4gMCA/IE1hdGgubWluKG5vZGVDb3VudCAvIDEwMCwgMSkgOiAwO1xufVxuXG5leHBvcnQgZnVuY3Rpb24gZ2V0Q29sbGlzaW9uUGFkZGluZyhub2RlOiBhbnkpOiBudW1iZXIge1xuICBpZiAobm9kZSAmJiBub2RlLnR5cGUgPT09ICdEb21haW4nKSB7XG4gICAgcmV0dXJuIENPTExJU0lPTl9QQURESU5HLkRvbWFpbjtcbiAgfVxuXG4gIGlmIChub2RlICYmIG5vZGUudHlwZSA9PT0gJ1N1YmRvbWFpbicpIHtcbiAgICByZXR1cm4gQ09MTElTSU9OX1BBRERJTkcuU3ViZG9tYWluO1xuICB9XG5cbiAgcmV0dXJuIENPTExJU0lPTl9QQURESU5HLmRlZmF1bHQ7XG59XG5cbmV4cG9ydCBmdW5jdGlvbiBjb21wdXRlSGFsZkRpYWdvbmFsKG5vZGU6IGFueSk6IG51bWJlciB7XG4gIGlmICghbm9kZSkge1xuICAgIHJldHVybiBSRUdVTEFSX05PREVfQkFTRV9TSVpFIC8gMjtcbiAgfVxuXG4gIGNvbnN0IHdpZHRoVmFsdWUgPSB0eXBlb2Ygbm9kZS53aWR0aCA9PT0gJ251bWJlcicgPyBub2RlLndpZHRoIDogdW5kZWZpbmVkO1xuICBjb25zdCBoZWlnaHRWYWx1ZSA9IHR5cGVvZiBub2RlLmhlaWdodCA9PT0gJ251bWJlcicgPyBub2RlLmhlaWdodCA6IHVuZGVmaW5lZDtcblxuICBpZiAod2lkdGhWYWx1ZSAhPT0gdW5kZWZpbmVkICYmIGhlaWdodFZhbHVlICE9PSB1bmRlZmluZWQpIHtcbiAgICBjb25zdCBoYWxmV2lkdGggPSB3aWR0aFZhbHVlIC8gMjtcbiAgICBjb25zdCBoYWxmSGVpZ2h0ID0gaGVpZ2h0VmFsdWUgLyAyO1xuICAgIHJldHVybiBNYXRoLnNxcnQoKGhhbGZXaWR0aCAqIGhhbGZXaWR0aCkgKyAoaGFsZkhlaWdodCAqIGhhbGZIZWlnaHQpKTtcbiAgfVxuXG4gIGNvbnN0IGZhbGxiYWNrU2l6ZSA9IHR5cGVvZiBub2RlLnBoeXNpY3NXZWlnaHQgPT09ICdudW1iZXInID8gbm9kZS5waHlzaWNzV2VpZ2h0IDogUkVHVUxBUl9OT0RFX0JBU0VfU0laRTtcbiAgcmV0dXJuIGZhbGxiYWNrU2l6ZSAvIDI7XG59XG5cbmV4cG9ydCBmdW5jdGlvbiBjb21wdXRlQ29sbGlzaW9uUmFkaXVzKG5vZGU6IGFueSk6IG51bWJlciB7XG4gIGNvbnN0IHBhZGRpbmcgPSBnZXRDb2xsaXNpb25QYWRkaW5nKG5vZGUpO1xuXG4gIGlmICghbm9kZSkge1xuICAgIHJldHVybiAoUkVHVUxBUl9OT0RFX0JBU0VfU0laRSAvIDIpICsgcGFkZGluZztcbiAgfVxuXG4gIGNvbnN0IHdpZHRoVmFsdWUgPSB0eXBlb2Ygbm9kZS53aWR0aCA9PT0gJ251bWJlcicgPyBub2RlLndpZHRoIDogdW5kZWZpbmVkO1xuICBjb25zdCBoZWlnaHRWYWx1ZSA9IHR5cGVvZiBub2RlLmhlaWdodCA9PT0gJ251bWJlcicgPyBub2RlLmhlaWdodCA6IHVuZGVmaW5lZDtcblxuICBpZiAod2lkdGhWYWx1ZSAhPT0gdW5kZWZpbmVkICYmIGhlaWdodFZhbHVlICE9PSB1bmRlZmluZWQpIHtcbiAgICBjb25zdCBsYXJnZXJIYWxmID0gd2lkdGhWYWx1ZSA+IGhlaWdodFZhbHVlID8gd2lkdGhWYWx1ZSAvIDIgOiBoZWlnaHRWYWx1ZSAvIDI7XG4gICAgcmV0dXJuIGxhcmdlckhhbGYgKyBwYWRkaW5nO1xuICB9XG5cbiAgY29uc3QgZmFsbGJhY2tTaXplID0gdHlwZW9mIG5vZGUucGh5c2ljc1dlaWdodCA9PT0gJ251bWJlcicgPyBub2RlLnBoeXNpY3NXZWlnaHQgOiBSRUdVTEFSX05PREVfQkFTRV9TSVpFO1xuICByZXR1cm4gKGZhbGxiYWNrU2l6ZSAvIDIpICsgcGFkZGluZztcbn1cblxuZXhwb3J0IGZ1bmN0aW9uIGNvbXB1dGVMaW5rRGlzdGFuY2UoXG4gIHNvdXJjZTogYW55LFxuICB0YXJnZXQ6IGFueSxcbiAgbGFiZWxXaWR0aD86IG51bWJlcixcbiAgbm9kZUNvdW50PzogbnVtYmVyXG4pOiBudW1iZXIge1xuICBpZiAoIXNvdXJjZSB8fCAhdGFyZ2V0KSB7XG4gICAgcmV0dXJuIDE1MDtcbiAgfVxuXG4gIGNvbnN0IHNvdXJjZVJhZGl1cyA9IGNvbXB1dGVIYWxmRGlhZ29uYWwoc291cmNlKTtcbiAgY29uc3QgdGFyZ2V0UmFkaXVzID0gY29tcHV0ZUhhbGZEaWFnb25hbCh0YXJnZXQpO1xuXG4gIGxldCBwYWRkaW5nID0gTElOS19QQURESU5HLmRlZmF1bHQ7XG5cbiAgY29uc3Qgc291cmNlVHlwZSA9IHNvdXJjZS50eXBlO1xuICBjb25zdCB0YXJnZXRUeXBlID0gdGFyZ2V0LnR5cGU7XG5cbiAgY29uc3QgaXNEb21haW5Eb21haW4gPSBzb3VyY2VUeXBlID09PSAnRG9tYWluJyAmJiB0YXJnZXRUeXBlID09PSAnRG9tYWluJztcbiAgY29uc3QgaXNEb21haW5PdGhlciA9ICFpc0RvbWFpbkRvbWFpbiAmJiAoc291cmNlVHlwZSA9PT0gJ0RvbWFpbicgfHwgdGFyZ2V0VHlwZSA9PT0gJ0RvbWFpbicpO1xuICBjb25zdCBpc1N1YmRvbWFpblByZXNlbnQgPSBzb3VyY2VUeXBlID09PSAnU3ViZG9tYWluJyB8fCB0YXJnZXRUeXBlID09PSAnU3ViZG9tYWluJztcblxuICBpZiAoaXNEb21haW5Eb21haW4pIHtcbiAgICBwYWRkaW5nID0gTElOS19QQURESU5HLmRvbWFpbkRvbWFpbjtcbiAgfSBlbHNlIGlmIChpc0RvbWFpbk90aGVyKSB7XG4gICAgcGFkZGluZyA9IExJTktfUEFERElORy5kb21haW5PdGhlcjtcbiAgfSBlbHNlIGlmIChpc1N1YmRvbWFpblByZXNlbnQpIHtcbiAgICBwYWRkaW5nID0gTElOS19QQURESU5HLnN1YmRvbWFpbk90aGVyO1xuICB9XG5cbiAgbGV0IGRpc3RhbmNlID0gc291cmNlUmFkaXVzICsgdGFyZ2V0UmFkaXVzICsgcGFkZGluZztcblxuICBpZiAoaXNEb21haW5Eb21haW4gJiYgdHlwZW9mIGxhYmVsV2lkdGggPT09ICdudW1iZXInICYmIGxhYmVsV2lkdGggPiAwKSB7XG4gICAgZGlzdGFuY2UgKz0gbGFiZWxXaWR0aDtcbiAgfVxuXG4gIGxldCBzcGFyc2VCb29zdCA9IDA7XG4gIGlmICh0eXBlb2Ygbm9kZUNvdW50ID09PSAnbnVtYmVyJyAmJiBub2RlQ291bnQgPCA1MCkge1xuICAgIGNvbnN0IGRlbnNpdHkgPSBnZXREZW5zaXR5RmFjdG9yKG5vZGVDb3VudCk7XG4gICAgc3BhcnNlQm9vc3QgPSBwYWRkaW5nICogKDEgLSBkZW5zaXR5KSAqIDAuOTtcbiAgfVxuXG4gIHJldHVybiBkaXN0YW5jZSArIHNwYXJzZUJvb3N0O1xufVxuXG5leHBvcnQgZnVuY3Rpb24gY29tcHV0ZUNoYXJnZVN0cmVuZ3RoKG5vZGU6IGFueSwgbm9kZUNvdW50OiBudW1iZXIpOiBudW1iZXIge1xuICAvLyBDUklUSUNBTDogRnJvemVuIG5vZGVzIChoaWRkZW4gYnkgRm9jdXMgTW9kZSkgc2hvdWxkIG5vdCBleGVydCBjaGFyZ2UgZm9yY2VcbiAgLy8gVGhleSBoYXZlIGZ4L2Z5IHNldCB0byBsb2NrIHBvc2l0aW9uLCBidXQgc3RpbGwgcGFydGljaXBhdGUgaW4gc2ltdWxhdGlvblxuICAvLyBJZiB0aGV5IHJlcGVsIGJ1dCBkb24ndCBhdHRyYWN0IChubyBsaW5rcyksIHRoZXkgcHVzaCB2aXNpYmxlIG5vZGVzIGF3YXlcbiAgY29uc3QgaXNGcm96ZW4gPSBub2RlICYmIG5vZGUuaXNIaWRkZW5CeUZvY3VzTW9kZSAmJiBub2RlLmZ4ICE9PSBudWxsICYmIG5vZGUuZnggIT09IHVuZGVmaW5lZDtcbiAgaWYgKGlzRnJvemVuKSB7XG4gICAgLy8gUmV0dXJuIHZlcnkgd2VhayBjaGFyZ2UgdG8gcHJldmVudCBhc3ltbWV0cmljIHJlcHVsc2lvblxuICAgIHJldHVybiAtMTsgLy8gTmVhci16ZXJvIHJlcHVsc2lvbiBmb3IgZnJvemVuIG5vZGVzXG4gIH1cblxuICBjb25zdCBkZW5zaXR5ID0gZ2V0RGVuc2l0eUZhY3Rvcihub2RlQ291bnQpO1xuXG4gIGxldCBiYXNlQ2hhcmdlID0gQkFTRV9DSEFSR0UuZGVmYXVsdDtcbiAgaWYgKG5vZGUgJiYgbm9kZS50eXBlID09PSAnRG9tYWluJykge1xuICAgIGJhc2VDaGFyZ2UgPSBCQVNFX0NIQVJHRS5Eb21haW47XG4gIH0gZWxzZSBpZiAobm9kZSAmJiBub2RlLnR5cGUgPT09ICdTdWJkb21haW4nKSB7XG4gICAgYmFzZUNoYXJnZSA9IEJBU0VfQ0hBUkdFLlN1YmRvbWFpbjtcbiAgfVxuXG4gIGNvbnN0IGRlbnNpdHlNdWx0aXBsaWVyID0gMS4zIC0gKGRlbnNpdHkgKiAwLjMpO1xuXG4gIGNvbnN0IHZpc2libGVEZWdyZWUgPSBub2RlICYmIHR5cGVvZiBub2RlLnZpc2libGVEZWdyZWUgPT09ICdudW1iZXInID8gbm9kZS52aXNpYmxlRGVncmVlIDogMDtcbiAgY29uc3QgZGVncmVlQm9vc3QgPSAxICsgKE1hdGgubG9nMTAodmlzaWJsZURlZ3JlZSArIDEpICogMC4xKTtcblxuICByZXR1cm4gYmFzZUNoYXJnZSAqIGRlbnNpdHlNdWx0aXBsaWVyICogZGVncmVlQm9vc3Q7XG59XG5cbmV4cG9ydCBmdW5jdGlvbiBjb21wdXRlTGlua1N0cmVuZ3RoKFxuICBzb3VyY2U6IGFueSxcbiAgdGFyZ2V0OiBhbnksXG4gIG5vZGVDb3VudDogbnVtYmVyXG4pOiBudW1iZXIge1xuICBjb25zdCBkZW5zaXR5ID0gZ2V0RGVuc2l0eUZhY3Rvcihub2RlQ291bnQpO1xuXG4gIGxldCBiYXNlU3RyZW5ndGggPSBCQVNFX0xJTktfU1RSRU5HVEguZGVmYXVsdDtcbiAgY29uc3Qgc291cmNlVHlwZSA9IHNvdXJjZSA/IHNvdXJjZS50eXBlIDogdW5kZWZpbmVkO1xuICBjb25zdCB0YXJnZXRUeXBlID0gdGFyZ2V0ID8gdGFyZ2V0LnR5cGUgOiB1bmRlZmluZWQ7XG5cbiAgY29uc3QgaXNEb21haW5Eb21haW4gPSBzb3VyY2VUeXBlID09PSAnRG9tYWluJyAmJiB0YXJnZXRUeXBlID09PSAnRG9tYWluJztcbiAgY29uc3QgaXNEb21haW5PdGhlciA9ICFpc0RvbWFpbkRvbWFpbiAmJiAoc291cmNlVHlwZSA9PT0gJ0RvbWFpbicgfHwgdGFyZ2V0VHlwZSA9PT0gJ0RvbWFpbicpO1xuXG4gIGlmIChpc0RvbWFpbkRvbWFpbikge1xuICAgIGJhc2VTdHJlbmd0aCA9IEJBU0VfTElOS19TVFJFTkdUSC5kb21haW5Eb21haW47XG4gIH0gZWxzZSBpZiAoaXNEb21haW5PdGhlcikge1xuICAgIGJhc2VTdHJlbmd0aCA9IEJBU0VfTElOS19TVFJFTkdUSC5kb21haW5PdGhlcjtcbiAgfVxuXG4gIGNvbnN0IGRlbnNpdHlNdWx0aXBsaWVyID0gMC40ICsgKGRlbnNpdHkgKiAwLjMpO1xuXG4gIHJldHVybiBiYXNlU3RyZW5ndGggKiBkZW5zaXR5TXVsdGlwbGllcjtcbn1cblxuZXhwb3J0IGZ1bmN0aW9uIGNvbXB1dGVDZW50ZXJTdHJlbmd0aChub2RlQ291bnQ6IG51bWJlcik6IG51bWJlciB7XG4gIGNvbnN0IGRlbnNpdHkgPSBnZXREZW5zaXR5RmFjdG9yKG5vZGVDb3VudCk7XG4gIHJldHVybiBCQVNFX0NFTlRFUl9TVFJFTkdUSCAqICgxICsgKGRlbnNpdHkgKiAyKSk7XG59XG4iLCAiaW1wb3J0ICogYXMgZDMgZnJvbSAnZDMnO1xuaW1wb3J0IHsgUGh5c2ljc0NvbmZpZywgREVGQVVMVF9QSFlTSUNTX0NPTkZJRyB9IGZyb20gJy4uLy4uLy4uL3BoeXNpY3MvUGh5c2ljc0NvbmZpZyc7XG5pbXBvcnQge1xuICBjb21wdXRlQ2hhcmdlU3RyZW5ndGgsXG4gIGNvbXB1dGVDb2xsaXNpb25SYWRpdXMsXG4gIGNvbXB1dGVMaW5rRGlzdGFuY2UsXG4gIGNvbXB1dGVMaW5rU3RyZW5ndGgsXG4gIGNvbXB1dGVDZW50ZXJTdHJlbmd0aFxufSBmcm9tICcuL3BoeXNpY3NIZWxwZXJzJztcblxuaW50ZXJmYWNlIFdvcmtlck1lc3NhZ2Uge1xuICB0eXBlOiAnc3RhcnQnIHwgJ3VwZGF0ZU5vZGVQb3NpdGlvbnMnIHwgJ3N0b3AnIHwgJ3JlaGVhdCcgfCAnc3RhcnREcmFnJyB8ICd1cGRhdGVEcmFnUG9zaXRpb24nIHwgJ2VuZERyYWcnIHwgJ3N0YXJ0Q29udGFpbmVyRHJhZycgfCAndXBkYXRlQ29udGFpbmVyRHJhZycgfCAnZW5kQ29udGFpbmVyRHJhZycgfCAndXBkYXRlUGh5c2ljcyc7XG4gIG5vZGVzPzogYW55W107XG4gIGxpbmtzPzogYW55W107XG4gIHVwZGF0ZWROb2Rlcz86IEFycmF5PHsgaWQ6IHN0cmluZzsgZng/OiBudW1iZXI7IGZ5PzogbnVtYmVyIH0+O1xuICBjaGFuZ2VkTm9kZUlkcz86IHN0cmluZ1tdO1xuICB3aWR0aD86IG51bWJlcjtcbiAgaGVpZ2h0PzogbnVtYmVyO1xuICBhbHBoYT86IG51bWJlcjtcbiAgbm9kZUlkPzogc3RyaW5nO1xuICBmeD86IG51bWJlcjtcbiAgZnk/OiBudW1iZXI7XG4gIG5vZGVJZHM/OiBzdHJpbmdbXTtcbiAgZHg/OiBudW1iZXI7XG4gIGR5PzogbnVtYmVyO1xuICBhY3RpdmU/OiBib29sZWFuOyAvLyBBZGQgYWN0aXZlIGZsYWcgZm9yIGRyYWcgZXZlbnRzXG4gIGNvbmZpZz86IFBoeXNpY3NDb25maWc7IC8vIFBoeXNpY3MgY29uZmlndXJhdGlvblxuICBsYXlvdXRIaW50PzogJ2Z1bGwnIHwgJ2luY3JlbWVudGFsJyB8ICdmb2N1c01vZGVUb2dnbGUnOyAvLyBMYXlvdXQgaGludCBmb3Igc2ltdWxhdGlvbiByZXN0YXJ0XG59XG5cbmludGVyZmFjZSBXb3JrZXJSZXNwb25zZSB7XG4gIHR5cGU6ICd0aWNrJyB8ICdlbmQnIHwgJ2Vycm9yJztcbiAgbm9kZXM/OiBBcnJheTx7IGlkOiBzdHJpbmc7IHg6IG51bWJlcjsgeTogbnVtYmVyIH0+O1xuICBjaGFuZ2VkTm9kZUlkcz86IHN0cmluZ1tdO1xuICBlcnJvcj86IHN0cmluZztcbiAgaXNEZWx0YT86IGJvb2xlYW47IC8vIEZsYWcgdG8gaW5kaWNhdGUgZGVsdGEgdXBkYXRlXG59XG5cbmxldCBzaW11bGF0aW9uOiBkMy5TaW11bGF0aW9uPGFueSwgYW55PiB8IG51bGwgPSBudWxsO1xubGV0IGN1cnJlbnRQaHlzaWNzQ29uZmlnOiBQaHlzaWNzQ29uZmlnID0gREVGQVVMVF9QSFlTSUNTX0NPTkZJRztcblxuLy8gT1BUSU1JWkFUSU9OOiBEZWx0YSBwb3NpdGlvbiB0cmFja2luZyB0byByZWR1Y2Ugd29ya2VyIHBheWxvYWRcbmxldCBsYXN0U2VudFBvc2l0aW9uczogTWFwPHN0cmluZywgeyB4OiBudW1iZXI7IHk6IG51bWJlciB9PiA9IG5ldyBNYXAoKTtcbmNvbnN0IHBvc2l0aW9uQ2hhbmdlVGhyZXNob2xkID0gMS4wOyAvLyBwaXhlbHNcblxuLy8gQkFUQ0hJTkc6IEFjY3VtdWxhdGUgcG9zaXRpb24gdXBkYXRlcyB0byBzZW5kIGF0IGZpeGVkIGludGVydmFsc1xubGV0IHBvc2l0aW9uQnVmZmVyID0gbmV3IE1hcDxzdHJpbmcsIHsgeDogbnVtYmVyOyB5OiBudW1iZXIgfT4oKTtcbmxldCBtZXNzYWdlU2NoZWR1bGVkID0gZmFsc2U7XG5sZXQgYmF0Y2hUaW1lcjogYW55ID0gbnVsbDtcbmxldCBwaHlzaWNzQ29vbGRvd25UaW1lcjogYW55ID0gbnVsbDtcbmxldCBzaW11bGF0aW9uV2lkdGggPSA4MDA7XG5sZXQgc2ltdWxhdGlvbkhlaWdodCA9IDYwMDtcblxuLy8gT1BUSU1JWkFUSU9OOiBHZXQgb25seSBub2RlcyB0aGF0IGhhdmUgbW92ZWQgc2lnbmlmaWNhbnRseSBzaW5jZSBsYXN0IHVwZGF0ZVxuZnVuY3Rpb24gZ2V0Q2hhbmdlZE5vZGVzKG5vZGVzOiBhbnlbXSk6IEFycmF5PHsgaWQ6IHN0cmluZzsgeDogbnVtYmVyOyB5OiBudW1iZXIgfT4ge1xuICBjb25zdCBjaGFuZ2VkTm9kZXM6IEFycmF5PHsgaWQ6IHN0cmluZzsgeDogbnVtYmVyOyB5OiBudW1iZXIgfT4gPSBbXTtcbiAgXG4gIGZvciAoY29uc3Qgbm9kZSBvZiBub2Rlcykge1xuICAgIGNvbnN0IGxhc3RQb3MgPSBsYXN0U2VudFBvc2l0aW9ucy5nZXQobm9kZS5pZCk7XG4gICAgXG4gICAgaWYgKCFsYXN0UG9zIHx8IFxuICAgICAgICBNYXRoLmFicyhub2RlLnggLSBsYXN0UG9zLngpID4gcG9zaXRpb25DaGFuZ2VUaHJlc2hvbGQgfHxcbiAgICAgICAgTWF0aC5hYnMobm9kZS55IC0gbGFzdFBvcy55KSA+IHBvc2l0aW9uQ2hhbmdlVGhyZXNob2xkKSB7XG4gICAgICBcbiAgICAgIGNoYW5nZWROb2Rlcy5wdXNoKHsgaWQ6IG5vZGUuaWQsIHg6IG5vZGUueCwgeTogbm9kZS55IH0pO1xuICAgICAgbGFzdFNlbnRQb3NpdGlvbnMuc2V0KG5vZGUuaWQsIHsgeDogbm9kZS54LCB5OiBub2RlLnkgfSk7XG4gICAgfVxuICB9XG4gIFxuICByZXR1cm4gY2hhbmdlZE5vZGVzO1xufVxuXG4vLyBCQVRDSElORzogU2VuZCBhY2N1bXVsYXRlZCBwb3NpdGlvbiB1cGRhdGVzIGF0IGZpeGVkIGludGVydmFsc1xuZnVuY3Rpb24gc2VuZEJhdGNoZWRVcGRhdGUoKSB7XG4gIGlmIChwb3NpdGlvbkJ1ZmZlci5zaXplID4gMCkge1xuICAgIGNvbnN0IG5vZGVzID0gQXJyYXkuZnJvbShwb3NpdGlvbkJ1ZmZlci5lbnRyaWVzKCkpLm1hcCgoW2lkLCB7eCwgeX1dKSA9PiAoeyBpZCwgeCwgeSB9KSk7XG4gICAgY29uc3QgY2hhbmdlZE5vZGVJZHMgPSBub2Rlcy5tYXAobiA9PiBuLmlkKTtcbiAgICBcbiAgICBwb3N0TWVzc2FnZSh7XG4gICAgICB0eXBlOiAndGljaycsXG4gICAgICBub2RlcyxcbiAgICAgIGNoYW5nZWROb2RlSWRzLFxuICAgICAgaXNEZWx0YTogdHJ1ZVxuICAgIH0gYXMgV29ya2VyUmVzcG9uc2UpO1xuICAgIFxuICAgIC8vIENsZWFyIGJ1ZmZlciBhZnRlciBzZW5kaW5nXG4gICAgcG9zaXRpb25CdWZmZXIuY2xlYXIoKTtcbiAgfVxuICBtZXNzYWdlU2NoZWR1bGVkID0gZmFsc2U7XG4gIGJhdGNoVGltZXIgPSBudWxsO1xufVxuXG5cbi8vIEN1c3RvbSBmb3JjZSB0byBwcmV2ZW50IGxhYmVsIG92ZXJsYXBzIChjb3BpZWQgZnJvbSBmb3JjZUxheW91dC50cylcbmZ1bmN0aW9uIGxhYmVsU2VwYXJhdGlvbkZvcmNlKCkge1xuICBsZXQgbm9kZXM6IGFueVtdO1xuICBsZXQgc3RyZW5ndGggPSAwLjU7XG4gIFxuICBmdW5jdGlvbiBmb3JjZShhbHBoYTogbnVtYmVyKSB7XG4gICAgaWYgKCFub2RlcykgcmV0dXJuO1xuICAgIFxuICAgIGZvciAobGV0IGkgPSAwOyBpIDwgbm9kZXMubGVuZ3RoOyBpKyspIHtcbiAgICAgIGZvciAobGV0IGogPSBpICsgMTsgaiA8IG5vZGVzLmxlbmd0aDsgaisrKSB7XG4gICAgICAgIGNvbnN0IG5vZGVBID0gbm9kZXNbaV07XG4gICAgICAgIGNvbnN0IG5vZGVCID0gbm9kZXNbal07XG4gICAgICAgIFxuICAgICAgICBjb25zdCByYWRpdXNBID0gbm9kZUEubGV2ZWwgPT09IDEgPyAxMiA6IG5vZGVBLmxldmVsID09PSAyID8gOCA6IDU7XG4gICAgICAgIGNvbnN0IHJhZGl1c0IgPSBub2RlQi5sZXZlbCA9PT0gMSA/IDEyIDogbm9kZUIubGV2ZWwgPT09IDIgPyA4IDogNTtcbiAgICAgICAgY29uc3QgZm9udFNpemVBID0gbm9kZUEubGV2ZWwgPT09IDEgPyAxNiA6IG5vZGVBLmxldmVsID09PSAyID8gMTMgOiAxMTtcbiAgICAgICAgY29uc3QgZm9udFNpemVCID0gbm9kZUIubGV2ZWwgPT09IDEgPyAxNiA6IG5vZGVCLmxldmVsID09PSAyID8gMTMgOiAxMTtcbiAgICAgICAgY29uc3QgbGFiZWxPZmZzZXRBID0gTWF0aC5tYXgoOCwgZm9udFNpemVBICogMC44KTtcbiAgICAgICAgY29uc3QgbGFiZWxPZmZzZXRCID0gTWF0aC5tYXgoOCwgZm9udFNpemVCICogMC44KTtcbiAgICAgICAgY29uc3QgbGFiZWxBWSA9IG5vZGVBLnkgKyByYWRpdXNBICsgbGFiZWxPZmZzZXRBO1xuICAgICAgICBjb25zdCBsYWJlbEJZID0gbm9kZUIueSArIHJhZGl1c0IgKyBsYWJlbE9mZnNldEI7XG4gICAgICAgIFxuICAgICAgICBjb25zdCBsYWJlbEFXaWR0aCA9IChub2RlQS5uYW1lIHx8IG5vZGVBLmlkIHx8ICcnKS5sZW5ndGggKiAobm9kZUEubGV2ZWwgPT09IDEgPyAxNiA6IG5vZGVBLmxldmVsID09PSAyID8gMTMgOiAxMSkgKiAwLjY7XG4gICAgICAgIGNvbnN0IGxhYmVsQldpZHRoID0gKG5vZGVCLm5hbWUgfHwgbm9kZUIuaWQgfHwgJycpLmxlbmd0aCAqIChub2RlQi5sZXZlbCA9PT0gMSA/IDE2IDogbm9kZUIubGV2ZWwgPT09IDIgPyAxMyA6IDExKSAqIDAuNjtcbiAgICAgICAgXG4gICAgICAgIGNvbnN0IGR4ID0gbm9kZUIueCAtIG5vZGVBLng7XG4gICAgICAgIGNvbnN0IGR5ID0gbGFiZWxCWSAtIGxhYmVsQVk7XG4gICAgICAgIGNvbnN0IGRpc3RhbmNlID0gTWF0aC5zcXJ0KGR4ICogZHggKyBkeSAqIGR5KTtcbiAgICAgICAgY29uc3QgbWluRGlzdGFuY2UgPSAobGFiZWxBV2lkdGggKyBsYWJlbEJXaWR0aCkgLyAyICsgMTA7XG4gICAgICAgIFxuICAgICAgICBpZiAoZGlzdGFuY2UgPCBtaW5EaXN0YW5jZSAmJiBNYXRoLmFicyhkeSkgPCAyMCkge1xuICAgICAgICAgIGNvbnN0IGZvcmNlID0gKG1pbkRpc3RhbmNlIC0gZGlzdGFuY2UpIC8gbWluRGlzdGFuY2UgKiBhbHBoYSAqIHN0cmVuZ3RoO1xuICAgICAgICAgIGNvbnN0IGZ4ID0gKGR4IC8gZGlzdGFuY2UpICogZm9yY2U7XG4gICAgICAgICAgY29uc3QgZnkgPSAoZHkgLyBkaXN0YW5jZSkgKiBmb3JjZTtcbiAgICAgICAgICBcbiAgICAgICAgICBub2RlQS52eCAtPSBmeDtcbiAgICAgICAgICBub2RlQS52eSAtPSBmeTtcbiAgICAgICAgICBub2RlQi52eCArPSBmeDtcbiAgICAgICAgICBub2RlQi52eSArPSBmeTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgfVxuICBcbiAgZm9yY2UuaW5pdGlhbGl6ZSA9IGZ1bmN0aW9uKG46IGFueVtdKSB7XG4gICAgbm9kZXMgPSBuO1xuICB9O1xuICBcbiAgZm9yY2Uuc3RyZW5ndGggPSBmdW5jdGlvbihfOiBudW1iZXIpIHtcbiAgICByZXR1cm4gYXJndW1lbnRzLmxlbmd0aCA/IChzdHJlbmd0aCA9IF8sIGZvcmNlKSA6IHN0cmVuZ3RoO1xuICB9O1xuICBcbiAgcmV0dXJuIGZvcmNlO1xufVxuXG5mdW5jdGlvbiBjcmVhdGVXb3JrZXJTaW11bGF0aW9uKHdpZHRoOiBudW1iZXIsIGhlaWdodDogbnVtYmVyKTogZDMuU2ltdWxhdGlvbjxhbnksIGFueT4ge1xuICBjb25zdCBjb25maWcgPSBjdXJyZW50UGh5c2ljc0NvbmZpZztcbiAgc2ltdWxhdGlvbldpZHRoID0gd2lkdGg7XG4gIHNpbXVsYXRpb25IZWlnaHQgPSBoZWlnaHQ7XG4gIGNvbnN0IHNpbSA9IGQzLmZvcmNlU2ltdWxhdGlvbigpXG4gICAgLmFscGhhRGVjYXkoY29uZmlnLmFscGhhRGVjYXkpXG4gICAgLnZlbG9jaXR5RGVjYXkoY29uZmlnLnZlbG9jaXR5RGVjYXkpXG4gICAgLmZvcmNlKCdjaGFyZ2UnLCBkMy5mb3JjZU1hbnlCb2R5KClcbiAgICAgIC5zdHJlbmd0aCgobm9kZTogYW55KSA9PiB7XG4gICAgICAgIGNvbnN0IG5vZGVzID0gc2ltLm5vZGVzKCk7XG4gICAgICAgIHJldHVybiBjb21wdXRlQ2hhcmdlU3RyZW5ndGgobm9kZSwgbm9kZXMubGVuZ3RoKTtcbiAgICAgIH0pXG4gICAgICAuZGlzdGFuY2VNYXgoY29uZmlnLmNoYXJnZURpc3RhbmNlTWF4KVxuICAgICAgLnRoZXRhKGNvbmZpZy5jaGFyZ2VUaGV0YSkpXG4gICAgLmZvcmNlKCdjZW50ZXInLCBkMy5mb3JjZUNlbnRlcih3aWR0aCAvIDIsIGhlaWdodCAvIDIpKVxuICAgIC5mb3JjZSgneCcsIGQzLmZvcmNlWCh3aWR0aCAvIDIpLnN0cmVuZ3RoKCgpID0+IHtcbiAgICAgIGNvbnN0IG5vZGVzID0gc2ltLm5vZGVzKCk7XG4gICAgICByZXR1cm4gY29tcHV0ZUNlbnRlclN0cmVuZ3RoKG5vZGVzLmxlbmd0aCk7XG4gICAgfSkpXG4gICAgLmZvcmNlKCd5JywgZDMuZm9yY2VZKGhlaWdodCAvIDIpLnN0cmVuZ3RoKCgpID0+IHtcbiAgICAgIGNvbnN0IG5vZGVzID0gc2ltLm5vZGVzKCk7XG4gICAgICByZXR1cm4gY29tcHV0ZUNlbnRlclN0cmVuZ3RoKG5vZGVzLmxlbmd0aCk7XG4gICAgfSkpXG4gICAgLmZvcmNlKCdjb2xsaXNpb24nLCBkMy5mb3JjZUNvbGxpZGUoKS5yYWRpdXMoKG5vZGU6IGFueSkgPT4gY29tcHV0ZUNvbGxpc2lvblJhZGl1cyhub2RlKSkpO1xuXG4gIHJldHVybiBzaW07XG59XG5cbmZ1bmN0aW9uIGFwcGx5UGh5c2ljc0NvbmZpZ1RvU2ltdWxhdGlvbihzaW06IGQzLlNpbXVsYXRpb248YW55LCBhbnk+LCBjb25maWc6IFBoeXNpY3NDb25maWcpOiB2b2lkIHtcbiAgc2ltXG4gICAgLmFscGhhRGVjYXkoY29uZmlnLmFscGhhRGVjYXkpXG4gICAgLnZlbG9jaXR5RGVjYXkoY29uZmlnLnZlbG9jaXR5RGVjYXkpO1xuXG4gIGNvbnN0IGNoYXJnZUZvcmNlID0gc2ltLmZvcmNlKCdjaGFyZ2UnKSBhcyBkMy5Gb3JjZU1hbnlCb2R5PGFueT47XG4gIGlmIChjaGFyZ2VGb3JjZSkge1xuICAgIGNoYXJnZUZvcmNlXG4gICAgICAuc3RyZW5ndGgoKG5vZGU6IGFueSkgPT4ge1xuICAgICAgICBjb25zdCBub2RlcyA9IHNpbS5ub2RlcygpO1xuICAgICAgICByZXR1cm4gY29tcHV0ZUNoYXJnZVN0cmVuZ3RoKG5vZGUsIG5vZGVzLmxlbmd0aCk7XG4gICAgICB9KVxuICAgICAgLmRpc3RhbmNlTWF4KGNvbmZpZy5jaGFyZ2VEaXN0YW5jZU1heClcbiAgICAgIC50aGV0YShjb25maWcuY2hhcmdlVGhldGEpO1xuICB9XG5cbiAgY29uc3QgY29sbGlzaW9uRm9yY2UgPSBzaW0uZm9yY2UoJ2NvbGxpc2lvbicpIGFzIGQzLkZvcmNlQ29sbGlkZTxhbnk+O1xuICBpZiAoY29sbGlzaW9uRm9yY2UpIHtcbiAgICBjb2xsaXNpb25Gb3JjZS5yYWRpdXMoKG5vZGU6IGFueSkgPT4gY29tcHV0ZUNvbGxpc2lvblJhZGl1cyhub2RlKSk7XG4gIH1cblxuICBjb25zdCBsaW5rRm9yY2UgPSBzaW0uZm9yY2UoJ2xpbmsnKSBhcyBkMy5Gb3JjZUxpbms8YW55LCBhbnk+O1xuICBpZiAobGlua0ZvcmNlKSB7XG4gICAgbGlua0ZvcmNlXG4gICAgICAuc3RyZW5ndGgoKGxpbms6IGFueSkgPT4ge1xuICAgICAgICBjb25zdCBub2RlcyA9IHNpbS5ub2RlcygpO1xuICAgICAgICBjb25zdCBzb3VyY2UgPSB0eXBlb2YgbGluay5zb3VyY2UgPT09ICdvYmplY3QnID8gbGluay5zb3VyY2UgOiBmaW5kTm9kZUJ5SWQobm9kZXMsIGxpbmsuc291cmNlKTtcbiAgICAgICAgY29uc3QgdGFyZ2V0ID0gdHlwZW9mIGxpbmsudGFyZ2V0ID09PSAnb2JqZWN0JyA/IGxpbmsudGFyZ2V0IDogZmluZE5vZGVCeUlkKG5vZGVzLCBsaW5rLnRhcmdldCk7XG4gICAgICAgIHJldHVybiBjb21wdXRlTGlua1N0cmVuZ3RoKHNvdXJjZSwgdGFyZ2V0LCBub2Rlcy5sZW5ndGgpO1xuICAgICAgfSlcbiAgICAgIC5kaXN0YW5jZSgobGluazogYW55KSA9PiB7XG4gICAgICAgIGNvbnN0IG5vZGVzID0gc2ltLm5vZGVzKCk7XG4gICAgICAgIGNvbnN0IHNvdXJjZSA9IHR5cGVvZiBsaW5rLnNvdXJjZSA9PT0gJ29iamVjdCcgPyBsaW5rLnNvdXJjZSA6IGZpbmROb2RlQnlJZChub2RlcywgbGluay5zb3VyY2UpO1xuICAgICAgICBjb25zdCB0YXJnZXQgPSB0eXBlb2YgbGluay50YXJnZXQgPT09ICdvYmplY3QnID8gbGluay50YXJnZXQgOiBmaW5kTm9kZUJ5SWQobm9kZXMsIGxpbmsudGFyZ2V0KTtcbiAgICAgICAgY29uc3QgbGFiZWxXaWR0aCA9IHR5cGVvZiBsaW5rLmxhYmVsV2lkdGggPT09ICdudW1iZXInID8gbGluay5sYWJlbFdpZHRoIDogdW5kZWZpbmVkO1xuICAgICAgICByZXR1cm4gY29tcHV0ZUxpbmtEaXN0YW5jZShzb3VyY2UsIHRhcmdldCwgbGFiZWxXaWR0aCwgbm9kZXMubGVuZ3RoKTtcbiAgICAgIH0pO1xuICB9XG5cbiAgc2ltLmZvcmNlKCd4JywgZDMuZm9yY2VYKHNpbXVsYXRpb25XaWR0aCAvIDIpLnN0cmVuZ3RoKCgpID0+IHtcbiAgICBjb25zdCBub2RlcyA9IHNpbS5ub2RlcygpO1xuICAgIHJldHVybiBjb21wdXRlQ2VudGVyU3RyZW5ndGgobm9kZXMubGVuZ3RoKTtcbiAgfSkpO1xuXG4gIHNpbS5mb3JjZSgneScsIGQzLmZvcmNlWShzaW11bGF0aW9uSGVpZ2h0IC8gMikuc3RyZW5ndGgoKCkgPT4ge1xuICAgIGNvbnN0IG5vZGVzID0gc2ltLm5vZGVzKCk7XG4gICAgcmV0dXJuIGNvbXB1dGVDZW50ZXJTdHJlbmd0aChub2Rlcy5sZW5ndGgpO1xuICB9KSk7XG59XG5cbi8qXG4vLyBERVBSRUNBVEVEOiBDb3B5IG9mIGxhYmVsIHNlcGFyYXRpb24gZm9yY2UgZnJvbSBtYWluIHRocmVhZCAoa2VwdCBmb3IgcmVmZXJlbmNlKVxuZnVuY3Rpb24gd29ya2VyTGFiZWxTZXBhcmF0aW9uRm9yY2UoKSB7XG4gIGxldCBub2RlczogYW55W107XG4gIGxldCBzdHJlbmd0aCA9IDAuNTtcbiAgZnVuY3Rpb24gZm9yY2UoYWxwaGE6IG51bWJlcikge1xuICAgIGlmICghbm9kZXMpIHJldHVybjtcbiAgICBmb3IgKGxldCBpID0gMDsgaSA8IG5vZGVzLmxlbmd0aDsgaSsrKSB7XG4gICAgICBmb3IgKGxldCBqID0gaSArIDE7IGogPCBub2Rlcy5sZW5ndGg7IGorKykge1xuICAgICAgICBjb25zdCBub2RlQSA9IG5vZGVzW2ldO1xuICAgICAgICBjb25zdCBub2RlQiA9IG5vZGVzW2pdO1xuICAgICAgICBjb25zdCByYWRpdXNBID0gbm9kZUEubGV2ZWwgPT09IDEgPyAxMiA6IG5vZGVBLmxldmVsID09PSAyID8gOCA6IDU7XG4gICAgICAgIGNvbnN0IHJhZGl1c0IgPSBub2RlQi5sZXZlbCA9PT0gMSA/IDEyIDogbm9kZUIubGV2ZWwgPT09IDIgPyA4IDogNTtcbiAgICAgICAgY29uc3QgZm9udFNpemVBID0gbm9kZUEubGV2ZWwgPT09IDEgPyAxNiA6IG5vZGVBLmxldmVsID09PSAyID8gMTMgOiAxMTtcbiAgICAgICAgY29uc3QgZm9udFNpemVCID0gbm9kZUIubGV2ZWwgPT09IDEgPyAxNiA6IG5vZGVCLmxldmVsID09PSAyID8gMTMgOiAxMTtcbiAgICAgICAgY29uc3QgbGFiZWxPZmZzZXRBID0gTWF0aC5tYXgoOCwgZm9udFNpemVBICogMC44KTtcbiAgICAgICAgY29uc3QgbGFiZWxPZmZzZXRCID0gTWF0aC5tYXgoOCwgZm9udFNpemVCICogMC44KTtcbiAgICAgICAgY29uc3QgbGFiZWxBWSA9IG5vZGVBLnkgKyByYWRpdXNBICsgbGFiZWxPZmZzZXRBO1xuICAgICAgICBjb25zdCBsYWJlbEJZID0gbm9kZUIueSArIHJhZGl1c0IgKyBsYWJlbE9mZnNldEI7XG4gICAgICAgIGNvbnN0IGxhYmVsQVdpZHRoID0gKG5vZGVBLm5hbWUgfHwgbm9kZUEuaWQgfHwgJycpLmxlbmd0aCAqIChub2RlQS5sZXZlbCA9PT0gMSA/IDE2IDogbm9kZUEubGV2ZWwgPT09IDIgPyAxMyA6IDExKSAqIDAuNjtcbiAgICAgICAgY29uc3QgbGFiZWxCV2lkdGggPSAobm9kZUIubmFtZSB8fCBub2RlQi5pZCB8fCAnJykubGVuZ3RoICogKG5vZGVCLmxldmVsID09PSAxID8gMTYgOiBub2RlQi5sZXZlbCA9PT0gMiA/IDEzIDogMTEpICogMC42O1xuICAgICAgICBjb25zdCBkeCA9IG5vZGVCLnggLSBub2RlQS54O1xuICAgICAgICBjb25zdCBkeSA9IGxhYmVsQlkgLSBsYWJlbEFZO1xuICAgICAgICBjb25zdCBkaXN0YW5jZSA9IE1hdGguc3FydChkeCAqIGR4ICsgZHkgKiBkeSk7XG4gICAgICAgIGNvbnN0IG1pbkRpc3RhbmNlID0gKGxhYmVsQVdpZHRoICsgbGFiZWxCV2lkdGgpIC8gMiArIDEwO1xuICAgICAgICBpZiAoZGlzdGFuY2UgPCBtaW5EaXN0YW5jZSAmJiBNYXRoLmFicyhkeSkgPCAyMCkge1xuICAgICAgICAgIGNvbnN0IGZvcmNlID0gKG1pbkRpc3RhbmNlIC0gZGlzdGFuY2UpIC8gbWluRGlzdGFuY2UgKiBhbHBoYSAqIHN0cmVuZ3RoO1xuICAgICAgICAgIGNvbnN0IGZ4ID0gKGR4IC8gZGlzdGFuY2UpICogZm9yY2U7XG4gICAgICAgICAgY29uc3QgZnkgPSAoZHkgLyBkaXN0YW5jZSkgKiBmb3JjZTtcbiAgICAgICAgICBub2RlQS52eCAtPSBmeDtcbiAgICAgICAgICBub2RlQS52eSAtPSBmeTtcbiAgICAgICAgICBub2RlQi52eCArPSBmeDtcbiAgICAgICAgICBub2RlQi52eSArPSBmeTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgfVxuICBmb3JjZS5pbml0aWFsaXplID0gZnVuY3Rpb24objogYW55W10pIHsgbm9kZXMgPSBuOyB9O1xuICBmb3JjZS5zdHJlbmd0aCA9IGZ1bmN0aW9uKF86IG51bWJlcikgeyByZXR1cm4gYXJndW1lbnRzLmxlbmd0aCA/IChzdHJlbmd0aCA9IF8sIGZvcmNlKSA6IHN0cmVuZ3RoOyB9O1xuICByZXR1cm4gZm9yY2U7XG59XG4qL1xuXG5mdW5jdGlvbiBjb25maWd1cmVGb3JjZVNpbXVsYXRpb25Gb3JMYXJnZUdyYXBocyhzaW11bGF0aW9uOiBkMy5TaW11bGF0aW9uPGFueSwgYW55Piwgd2lkdGg6IG51bWJlciwgaGVpZ2h0OiBudW1iZXIpOiB2b2lkIHtcbiAgc2ltdWxhdGlvbi5mb3JjZSgnbGluaycsIGQzLmZvcmNlTGluaygpLmlkKChkOiBhbnkpID0+IGQuaWQpXG4gICAgLnN0cmVuZ3RoKChsaW5rOiBhbnkpID0+IHtcbiAgICAgIGNvbnN0IG5vZGVzID0gc2ltdWxhdGlvbi5ub2RlcygpO1xuICAgICAgY29uc3Qgc291cmNlID0gdHlwZW9mIGxpbmsuc291cmNlID09PSAnb2JqZWN0JyA/IGxpbmsuc291cmNlIDogZmluZE5vZGVCeUlkKG5vZGVzLCBsaW5rLnNvdXJjZSk7XG4gICAgICBjb25zdCB0YXJnZXQgPSB0eXBlb2YgbGluay50YXJnZXQgPT09ICdvYmplY3QnID8gbGluay50YXJnZXQgOiBmaW5kTm9kZUJ5SWQobm9kZXMsIGxpbmsudGFyZ2V0KTtcbiAgICAgIHJldHVybiBjb21wdXRlTGlua1N0cmVuZ3RoKHNvdXJjZSwgdGFyZ2V0LCBub2Rlcy5sZW5ndGgpO1xuICAgIH0pXG4gICAgLmRpc3RhbmNlKChsaW5rOiBhbnkpID0+IHtcbiAgICAgIGNvbnN0IG5vZGVzID0gc2ltdWxhdGlvbi5ub2RlcygpO1xuICAgICAgY29uc3Qgc291cmNlID0gdHlwZW9mIGxpbmsuc291cmNlID09PSAnb2JqZWN0JyA/IGxpbmsuc291cmNlIDogZmluZE5vZGVCeUlkKG5vZGVzLCBsaW5rLnNvdXJjZSk7XG4gICAgICBjb25zdCB0YXJnZXQgPSB0eXBlb2YgbGluay50YXJnZXQgPT09ICdvYmplY3QnID8gbGluay50YXJnZXQgOiBmaW5kTm9kZUJ5SWQobm9kZXMsIGxpbmsudGFyZ2V0KTtcbiAgICAgIGNvbnN0IGxhYmVsV2lkdGggPSB0eXBlb2YgbGluay5sYWJlbFdpZHRoID09PSAnbnVtYmVyJyA/IGxpbmsubGFiZWxXaWR0aCA6IHVuZGVmaW5lZDtcbiAgICAgIHJldHVybiBjb21wdXRlTGlua0Rpc3RhbmNlKHNvdXJjZSwgdGFyZ2V0LCBsYWJlbFdpZHRoLCBub2Rlcy5sZW5ndGgpO1xuICAgIH0pKTtcblxuICBzaW11bGF0aW9uLmZvcmNlKCdjZW50ZXInLCBkMy5mb3JjZUNlbnRlcih3aWR0aCAvIDIsIGhlaWdodCAvIDIpKTtcblxuICBzaW11bGF0aW9uLmZvcmNlKCd4JywgZDMuZm9yY2VYKHdpZHRoIC8gMikuc3RyZW5ndGgoKCkgPT4ge1xuICAgIGNvbnN0IG5vZGVzID0gc2ltdWxhdGlvbi5ub2RlcygpO1xuICAgIHJldHVybiBjb21wdXRlQ2VudGVyU3RyZW5ndGgobm9kZXMubGVuZ3RoKTtcbiAgfSkpO1xuXG4gIHNpbXVsYXRpb24uZm9yY2UoJ3knLCBkMy5mb3JjZVkoaGVpZ2h0IC8gMikuc3RyZW5ndGgoKCkgPT4ge1xuICAgIGNvbnN0IG5vZGVzID0gc2ltdWxhdGlvbi5ub2RlcygpO1xuICAgIHJldHVybiBjb21wdXRlQ2VudGVyU3RyZW5ndGgobm9kZXMubGVuZ3RoKTtcbiAgfSkpO1xuXG4gIGNvbnN0IGNvbGxpc2lvbkZvcmNlID0gc2ltdWxhdGlvbi5mb3JjZSgnY29sbGlzaW9uJykgYXMgZDMuRm9yY2VDb2xsaWRlPGFueT47XG4gIGlmIChjb2xsaXNpb25Gb3JjZSkge1xuICAgIGNvbGxpc2lvbkZvcmNlLnJhZGl1cygobm9kZTogYW55KSA9PiBjb21wdXRlQ29sbGlzaW9uUmFkaXVzKG5vZGUpKTtcbiAgfSBlbHNlIHtcbiAgICBzaW11bGF0aW9uLmZvcmNlKCdjb2xsaXNpb24nLCBkMy5mb3JjZUNvbGxpZGUoKS5yYWRpdXMoKG5vZGU6IGFueSkgPT4gY29tcHV0ZUNvbGxpc2lvblJhZGl1cyhub2RlKSkpO1xuICB9XG59XG5cbi8vIExpc3RlbiBmb3IgbWVzc2FnZXMgZnJvbSBtYWluIHRocmVhZFxuc2VsZi5vbm1lc3NhZ2UgPSBmdW5jdGlvbihldmVudDogTWVzc2FnZUV2ZW50PFdvcmtlck1lc3NhZ2U+KSB7XG4gIGNvbnN0IHsgdHlwZSwgbm9kZXMsIGxpbmtzLCB1cGRhdGVkTm9kZXMsIHdpZHRoLCBoZWlnaHQsIGFscGhhLCBub2RlSWQsIGZ4LCBmeSwgbm9kZUlkcywgZHgsIGR5LCBhY3RpdmUsIGNvbmZpZywgbGF5b3V0SGludCB9ID0gZXZlbnQuZGF0YTtcbiAgXG4gIC8vIE9ubHkgbG9nIG5vbi10aWNrIG1lc3NhZ2VzIGFuZCBvY2Nhc2lvbmFsIHRpY2sgbWVzc2FnZXNcbiAgaWYgKHR5cGUgIT09ICd1cGRhdGVOb2RlUG9zaXRpb25zJyB8fCBNYXRoLnJhbmRvbSgpIDwgMC4xKSB7XG4gICAgLy8gV29ya2VyIG1lc3NhZ2U6ICR7dHlwZX0gKCR7bm9kZXM/Lmxlbmd0aCB8fCAwfSBub2RlcywgJHtsaW5rcz8ubGVuZ3RoIHx8IDB9IGxpbmtzKVxuICB9XG4gIFxuICB0cnkge1xuICAgIHN3aXRjaCAodHlwZSkge1xuICAgICAgY2FzZSAnc3RhcnQnOlxuICAgICAgICBcbiAgICAgICAgaWYgKCFub2RlcyB8fCAhbGlua3MgfHwgd2lkdGggPT09IHVuZGVmaW5lZCB8fCBoZWlnaHQgPT09IHVuZGVmaW5lZCkge1xuICAgICAgICAgIHBvc3RNZXNzYWdlKHtcbiAgICAgICAgICAgIHR5cGU6ICdlcnJvcicsXG4gICAgICAgICAgICBlcnJvcjogJ01pc3NpbmcgcmVxdWlyZWQgZGF0YSBmb3Igc2ltdWxhdGlvbiBzdGFydCdcbiAgICAgICAgICB9IGFzIFdvcmtlclJlc3BvbnNlKTtcbiAgICAgICAgICByZXR1cm47XG4gICAgICAgIH1cbiAgICAgICAgXG4gICAgICAgIC8vIE9QVElNSVpBVElPTjogQ2xlYXIgZGVsdGEgdHJhY2tpbmcgYW5kIGJhdGNoaW5nIG9uIHNpbXVsYXRpb24gcmVzdGFydFxuICAgICAgICBsYXN0U2VudFBvc2l0aW9ucy5jbGVhcigpO1xuICAgICAgICBwb3NpdGlvbkJ1ZmZlci5jbGVhcigpO1xuICAgICAgICBpZiAoYmF0Y2hUaW1lcikge1xuICAgICAgICAgIGNsZWFyVGltZW91dChiYXRjaFRpbWVyKTtcbiAgICAgICAgICBiYXRjaFRpbWVyID0gbnVsbDtcbiAgICAgICAgfVxuICAgICAgICBtZXNzYWdlU2NoZWR1bGVkID0gZmFsc2U7XG4gICAgICAgIFxuICAgICAgICAvLyBDcmVhdGUgbmV3IHNpbXVsYXRpb25cbiAgICAgICAgc2ltdWxhdGlvbiA9IGNyZWF0ZVdvcmtlclNpbXVsYXRpb24od2lkdGgsIGhlaWdodCk7XG4gICAgICAgIC8vIENyZWF0ZWQgc2ltdWxhdGlvbiBmb3IgJHtub2Rlcy5sZW5ndGh9IG5vZGVzXG4gICAgICAgIFxuICAgICAgICAvLyBTZXQgdXAgbm9kZXMgYW5kIGxpbmtzXG4gICAgICAgIHNpbXVsYXRpb24ubm9kZXMobm9kZXMpO1xuICAgICAgICBcbiAgICAgICAgY29uc3QgbGlua0ZvcmNlID0gZDMuZm9yY2VMaW5rKGxpbmtzKS5pZCgobGluazogYW55KSA9PiBsaW5rLmlkKVxuICAgICAgICAgIC5zdHJlbmd0aCgobGluazogYW55KSA9PiB7XG4gICAgICAgICAgICBjb25zdCBzaW1Ob2RlcyA9IHNpbXVsYXRpb24ubm9kZXMoKTtcbiAgICAgICAgICAgIGNvbnN0IHNvdXJjZSA9IHR5cGVvZiBsaW5rLnNvdXJjZSA9PT0gJ29iamVjdCcgPyBsaW5rLnNvdXJjZSA6IGZpbmROb2RlQnlJZChzaW1Ob2RlcywgbGluay5zb3VyY2UpO1xuICAgICAgICAgICAgY29uc3QgdGFyZ2V0ID0gdHlwZW9mIGxpbmsudGFyZ2V0ID09PSAnb2JqZWN0JyA/IGxpbmsudGFyZ2V0IDogZmluZE5vZGVCeUlkKHNpbU5vZGVzLCBsaW5rLnRhcmdldCk7XG4gICAgICAgICAgICByZXR1cm4gY29tcHV0ZUxpbmtTdHJlbmd0aChzb3VyY2UsIHRhcmdldCwgc2ltTm9kZXMubGVuZ3RoKTtcbiAgICAgICAgICB9KVxuICAgICAgICAgIC5kaXN0YW5jZSgobGluazogYW55KSA9PiB7XG4gICAgICAgICAgICBjb25zdCBzaW1Ob2RlcyA9IHNpbXVsYXRpb24ubm9kZXMoKTtcbiAgICAgICAgICAgIGNvbnN0IHNvdXJjZSA9IHR5cGVvZiBsaW5rLnNvdXJjZSA9PT0gJ29iamVjdCcgPyBsaW5rLnNvdXJjZSA6IGZpbmROb2RlQnlJZChzaW1Ob2RlcywgbGluay5zb3VyY2UpO1xuICAgICAgICAgICAgY29uc3QgdGFyZ2V0ID0gdHlwZW9mIGxpbmsudGFyZ2V0ID09PSAnb2JqZWN0JyA/IGxpbmsudGFyZ2V0IDogZmluZE5vZGVCeUlkKHNpbU5vZGVzLCBsaW5rLnRhcmdldCk7XG4gICAgICAgICAgICBjb25zdCBsYWJlbFdpZHRoID0gdHlwZW9mIGxpbmsubGFiZWxXaWR0aCA9PT0gJ251bWJlcicgPyBsaW5rLmxhYmVsV2lkdGggOiB1bmRlZmluZWQ7XG4gICAgICAgICAgICByZXR1cm4gY29tcHV0ZUxpbmtEaXN0YW5jZShzb3VyY2UsIHRhcmdldCwgbGFiZWxXaWR0aCwgc2ltTm9kZXMubGVuZ3RoKTtcbiAgICAgICAgICB9KTtcbiAgICAgICAgc2ltdWxhdGlvbi5mb3JjZSgnbGluaycsIGxpbmtGb3JjZSk7XG4gICAgICAgIFxuICAgICAgICAvLyBObyBzcGVjaWFsIGNvbmZpZ3VyYXRpb24gZm9yIGxhcmdlIGdyYXBocyAtIHVzZSBzYW1lIHBoeXNpY3MgZm9yIGFsbFxuICAgICAgICBcbiAgICAgICAgLy8gVXNlIExJVkUgc2ltdWxhdGlvbiBmb3IgYWxsIGdyYXBoIHNpemVzIHRvIGVuYWJsZSBkcmFnIGFuZCBjb250aW51b3VzIHVwZGF0ZXNcbiAgICAgICAge1xuICAgICAgICAgIC8vIFNldCB1cCB0aWNrIGhhbmRsZXIgd2l0aCBiYXRjaGluZ1xuICAgICAgICAgIGxldCB0aWNrQ291bnQgPSAwO1xuICAgICAgICAgIHNpbXVsYXRpb24ub24oJ3RpY2snLCAoKSA9PiB7XG4gICAgICAgICAgICBpZiAoIXNpbXVsYXRpb24pIHJldHVybjtcbiAgICAgICAgICAgIFxuICAgICAgICAgICAgdGlja0NvdW50Kys7XG4gICAgICAgICAgICBjb25zdCBub2RlcyA9IHNpbXVsYXRpb24ubm9kZXMoKTtcbiAgICAgICAgICAgIFxuICAgICAgICAgICAgLy8gTWluaW1hbCBsb2dnaW5nIGZvciBjcml0aWNhbCBzdGF0ZXMgb25seVxuICAgICAgICAgICAgaWYgKHRpY2tDb3VudCA8PSAzKSB7XG4gICAgICAgICAgICAgIC8vIFRpY2sgJHt0aWNrQ291bnR9LCBhbHBoYTogJHtzaW11bGF0aW9uLmFscGhhKCkudG9GaXhlZCgzKX1cbiAgICAgICAgICAgIH1cbiAgICAgICAgICAgIFxuICAgICAgICAgICAgLy8gQkFUQ0hJTkc6IEFjY3VtdWxhdGUgY2hhbmdlZCBub2RlcyBpbiBidWZmZXJcbiAgICAgICAgICAgIGNvbnN0IGNoYW5nZWROb2RlcyA9IGdldENoYW5nZWROb2Rlcyhub2Rlcyk7XG4gICAgICAgICAgICBjaGFuZ2VkTm9kZXMuZm9yRWFjaChub2RlID0+IHtcbiAgICAgICAgICAgICAgcG9zaXRpb25CdWZmZXIuc2V0KG5vZGUuaWQsIHsgeDogbm9kZS54LCB5OiBub2RlLnkgfSk7XG4gICAgICAgICAgICB9KTtcbiAgICAgICAgICAgIFxuICAgICAgICAgICAgLy8gU2NoZWR1bGUgYmF0Y2ggc2VuZCBpZiBub3QgYWxyZWFkeSBzY2hlZHVsZWRcbiAgICAgICAgICAgIGlmICghbWVzc2FnZVNjaGVkdWxlZCAmJiBwb3NpdGlvbkJ1ZmZlci5zaXplID4gMCkge1xuICAgICAgICAgICAgICBtZXNzYWdlU2NoZWR1bGVkID0gdHJ1ZTtcbiAgICAgICAgICAgICAgLy8gVXNlIHNldFRpbWVvdXQgZm9yIHJlbGlhYmxlIHRocm90dGxpbmcgaW4gd29ya2VyICgxNm1zID0gfjYwZnBzKVxuICAgICAgICAgICAgICBiYXRjaFRpbWVyID0gc2V0VGltZW91dChzZW5kQmF0Y2hlZFVwZGF0ZSwgMTYpO1xuICAgICAgICAgICAgfVxuICAgICAgICAgIH0pO1xuICAgICAgICAgIFxuICAgICAgICAgIHNpbXVsYXRpb24ub24oJ2VuZCcsICgpID0+IHtcbiAgICAgICAgICAgIC8vIEZsdXNoIGFueSByZW1haW5pbmcgYnVmZmVyZWQgcG9zaXRpb25zIGJlZm9yZSBlbmRpbmdcbiAgICAgICAgICAgIGlmIChwb3NpdGlvbkJ1ZmZlci5zaXplID4gMCkge1xuICAgICAgICAgICAgICBzZW5kQmF0Y2hlZFVwZGF0ZSgpO1xuICAgICAgICAgICAgfVxuICAgICAgICAgICAgcG9zdE1lc3NhZ2UoeyB0eXBlOiAnZW5kJyB9IGFzIFdvcmtlclJlc3BvbnNlKTtcbiAgICAgICAgICB9KTtcbiAgICAgICAgICBcbiAgICAgICAgICAvLyBVc2UgbGF5b3V0SGludCB0byBkZXRlcm1pbmUgdGhlIHJlc3RhcnQgYWxwaGFcbiAgICAgICAgICBpZiAobGF5b3V0SGludCA9PT0gJ2luY3JlbWVudGFsJykge1xuICAgICAgICAgICAgc2ltdWxhdGlvbi5hbHBoYSgwLjMpLnJlc3RhcnQoKTsgLy8gR2VudGxlIHJlaGVhdCBmb3IgaW5jcmVtZW50YWwgdXBkYXRlc1xuICAgICAgICAgIH0gZWxzZSBpZiAobGF5b3V0SGludCA9PT0gJ2ZvY3VzTW9kZVRvZ2dsZScpIHtcbiAgICAgICAgICAgIHNpbXVsYXRpb24uYWxwaGEoMC4xKS5yZXN0YXJ0KCk7IC8vIFZlcnkgZ2VudGxlIGZvciBGb2N1cyBNb2RlIHRvIHByZXZlbnQgZHJpZnRcbiAgICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgICAgc2ltdWxhdGlvbi5hbHBoYSgxKS5yZXN0YXJ0KCk7IC8vIEZ1bGwgcmVzdGFydCBmb3IgY29tcGxldGUgbGF5b3V0XG4gICAgICAgICAgfVxuICAgICAgICB9XG4gICAgICAgIGJyZWFrO1xuICAgICAgICBcbiAgICAgIGNhc2UgJ3N0YXJ0RHJhZyc6XG4gICAgICAgIGlmICghc2ltdWxhdGlvbikgcmV0dXJuO1xuICAgICAgICBcbiAgICAgICAgY29uc3Qgc3RhcnROb2RlID0gc2ltdWxhdGlvbi5ub2RlcygpLmZpbmQobiA9PiBuLmlkID09PSBub2RlSWQpO1xuICAgICAgICBpZiAoc3RhcnROb2RlKSB7XG4gICAgICAgICAgc3RhcnROb2RlLmZ4ID0gZng7XG4gICAgICAgICAgc3RhcnROb2RlLmZ5ID0gZnk7XG4gICAgICAgICAgLy8gQ1JJVElDQUwgRklYOiBBbHdheXMgZW5zdXJlIHNpbXVsYXRpb24gaGFzIGVuZXJneSBkdXJpbmcgZHJhZywgYnV0IHJlc3BlY3QgYWN0aXZlIHN0YXRlXG4gICAgICAgICAgaWYgKCFhY3RpdmUpIHtcbiAgICAgICAgICAgIHNpbXVsYXRpb24uYWxwaGFUYXJnZXQoMC4zKS5yZXN0YXJ0KCk7XG4gICAgICAgICAgICAvLyBTdGFydGVkIGRyYWcgZm9yIG5vZGUgJHtub2RlSWR9IHdpdGggcmVzdGFydFxuICAgICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgICAvLyBFdmVuIGlmIGFscmVhZHkgYWN0aXZlLCBlbnN1cmUgd2UgaGF2ZSBzb21lIGVuZXJneSBmb3IgZHJhZyByZXNwb25zaXZlbmVzc1xuICAgICAgICAgICAgc2ltdWxhdGlvbi5hbHBoYVRhcmdldCgwLjMpO1xuICAgICAgICAgICAgLy8gU3RhcnRlZCBkcmFnIGZvciBub2RlICR7bm9kZUlkfSB3aXRoIGFscGhhIHRhcmdldFxuICAgICAgICAgIH1cbiAgICAgICAgfVxuICAgICAgICBicmVhaztcbiAgICAgICAgXG4gICAgICBjYXNlICd1cGRhdGVEcmFnUG9zaXRpb24nOlxuICAgICAgICBpZiAoIXNpbXVsYXRpb24pIHJldHVybjtcbiAgICAgICAgXG4gICAgICAgIGNvbnN0IGRyYWdOb2RlID0gc2ltdWxhdGlvbi5ub2RlcygpLmZpbmQobiA9PiBuLmlkID09PSBub2RlSWQpO1xuICAgICAgICBpZiAoZHJhZ05vZGUpIHtcbiAgICAgICAgICBkcmFnTm9kZS5meCA9IGZ4O1xuICAgICAgICAgIGRyYWdOb2RlLmZ5ID0gZnk7XG4gICAgICAgICAgLy8gUEVSRk9STUFOQ0UgRklYOiBFbnN1cmUgc2ltdWxhdGlvbiBzdGF5cyBhY3RpdmUgZHVyaW5nIGRyYWcgdXBkYXRlc1xuICAgICAgICAgIGlmIChzaW11bGF0aW9uLmFscGhhKCkgPCAwLjAxKSB7XG4gICAgICAgICAgICBzaW11bGF0aW9uLmFscGhhKDAuMSk7IC8vIEJvb3N0IGFscGhhIGlmIHNpbXVsYXRpb24gaXMgZHlpbmdcbiAgICAgICAgICAgIC8vIEJvb3N0ZWQgYWxwaGEgZHVyaW5nIGRyYWc6ICR7c2ltdWxhdGlvbi5hbHBoYSgpLnRvRml4ZWQoMyl9XG4gICAgICAgICAgfVxuICAgICAgICB9XG4gICAgICAgIGJyZWFrO1xuICAgICAgICBcbiAgICAgIGNhc2UgJ2VuZERyYWcnOlxuICAgICAgICBpZiAoIXNpbXVsYXRpb24pIHJldHVybjtcbiAgICAgICAgXG4gICAgICAgIGNvbnN0IGVuZE5vZGUgPSBzaW11bGF0aW9uLm5vZGVzKCkuZmluZChuID0+IG4uaWQgPT09IG5vZGVJZCk7XG4gICAgICAgIGlmIChlbmROb2RlKSB7XG4gICAgICAgICAgZW5kTm9kZS5meCA9IG51bGw7XG4gICAgICAgICAgZW5kTm9kZS5meSA9IG51bGw7XG4gICAgICAgICAgXG4gICAgICAgICAgaWYgKCFhY3RpdmUpIHtcbiAgICAgICAgICAgIC8vIElmIHRoZSBzaW11bGF0aW9uIHdhcyBub3QgYWN0aXZlLCBqdXN0IGNvb2wgaXQgZG93bi5cbiAgICAgICAgICAgIHNpbXVsYXRpb24uYWxwaGFUYXJnZXQoMCk7XG4gICAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICAgIC8vIElmIGl0IHdhcyBhY3RpdmUsIGdpdmUgaXQgYSBnZW50bGUgbnVkZ2UgdG8gc2V0dGxlIG5pY2VseS5cbiAgICAgICAgICAgIHNpbXVsYXRpb24uYWxwaGFUYXJnZXQoMC4xKS5yZXN0YXJ0KCk7XG4gICAgICAgICAgICBzaW11bGF0aW9uLmFscGhhVGFyZ2V0KDApOyAvLyBUaGVuIGxldCBpdCBjb29sIGRvd24gYWdhaW4uXG4gICAgICAgICAgfVxuICAgICAgICAgIC8vIEVuZGVkIGRyYWcgZm9yIG5vZGUgJHtub2RlSWR9XG4gICAgICAgIH1cbiAgICAgICAgYnJlYWs7XG4gICAgICAgIFxuICAgICAgY2FzZSAnc3RhcnRDb250YWluZXJEcmFnJzpcbiAgICAgICAgaWYgKCFzaW11bGF0aW9uIHx8ICFub2RlSWRzKSByZXR1cm47XG4gICAgICAgIFxuICAgICAgICBjb25zdCBjb250YWluZXJOb2RlcyA9IHNpbXVsYXRpb24ubm9kZXMoKS5maWx0ZXIobiA9PiBub2RlSWRzLmluY2x1ZGVzKG4uaWQpKTtcbiAgICAgICAgY29udGFpbmVyTm9kZXMuZm9yRWFjaChub2RlID0+IHtcbiAgICAgICAgICBub2RlLmZ4ID0gbm9kZS54O1xuICAgICAgICAgIG5vZGUuZnkgPSBub2RlLnk7XG4gICAgICAgIH0pO1xuICAgICAgICBzaW11bGF0aW9uLmFscGhhVGFyZ2V0KDAuMykucmVzdGFydCgpO1xuICAgICAgICAvLyBTdGFydGVkIGNvbnRhaW5lciBkcmFnIGZvciAke2NvbnRhaW5lck5vZGVzLmxlbmd0aH0gbm9kZXNcbiAgICAgICAgYnJlYWs7XG4gICAgICAgIFxuICAgICAgY2FzZSAndXBkYXRlQ29udGFpbmVyRHJhZyc6XG4gICAgICAgIGlmICghc2ltdWxhdGlvbiB8fCAhbm9kZUlkcyB8fCBkeCA9PT0gdW5kZWZpbmVkIHx8IGR5ID09PSB1bmRlZmluZWQpIHJldHVybjtcbiAgICAgICAgXG4gICAgICAgIGNvbnN0IGRyYWdDb250YWluZXJOb2RlcyA9IHNpbXVsYXRpb24ubm9kZXMoKS5maWx0ZXIobiA9PiBub2RlSWRzLmluY2x1ZGVzKG4uaWQpKTtcbiAgICAgICAgZHJhZ0NvbnRhaW5lck5vZGVzLmZvckVhY2gobm9kZSA9PiB7XG4gICAgICAgICAgaWYgKG5vZGUuZnggIT09IG51bGwgJiYgbm9kZS5meSAhPT0gbnVsbCkge1xuICAgICAgICAgICAgbm9kZS5meCA9IChub2RlLmZ4IHx8IG5vZGUueCkgKyBkeDtcbiAgICAgICAgICAgIG5vZGUuZnkgPSAobm9kZS5meSB8fCBub2RlLnkpICsgZHk7XG4gICAgICAgICAgfVxuICAgICAgICB9KTtcbiAgICAgICAgLy8gRG9uJ3QgcmVzdGFydCAtIGp1c3QgbGV0IHNpbXVsYXRpb24gY29udGludWUgd2l0aCB1cGRhdGVkIHBvc2l0aW9uc1xuICAgICAgICAvLyBVcGRhdGVkICR7ZHJhZ0NvbnRhaW5lck5vZGVzLmxlbmd0aH0gbm9kZXMgYnkgZHg9JHtkeH0sIGR5PSR7ZHl9XG4gICAgICAgIGJyZWFrO1xuICAgICAgICBcbiAgICAgIGNhc2UgJ2VuZENvbnRhaW5lckRyYWcnOlxuICAgICAgICBpZiAoIXNpbXVsYXRpb24gfHwgIW5vZGVJZHMpIHJldHVybjtcbiAgICAgICAgXG4gICAgICAgIGNvbnN0IGVuZENvbnRhaW5lck5vZGVzID0gc2ltdWxhdGlvbi5ub2RlcygpLmZpbHRlcihuID0+IG5vZGVJZHMuaW5jbHVkZXMobi5pZCkpO1xuICAgICAgICBlbmRDb250YWluZXJOb2Rlcy5mb3JFYWNoKG5vZGUgPT4ge1xuICAgICAgICAgIG5vZGUuZnggPSBudWxsO1xuICAgICAgICAgIG5vZGUuZnkgPSBudWxsO1xuICAgICAgICB9KTtcbiAgICAgICAgc2ltdWxhdGlvbi5hbHBoYVRhcmdldCgwKTtcbiAgICAgICAgLy8gR2VudGxlIHJlaGVhdCB0byBzZXR0bGUgbm9kZXMgKHNhbWUgYXMgbWFpbiB0aHJlYWQpXG4gICAgICAgIHNpbXVsYXRpb24uYWxwaGEoMC4xKS5yZXN0YXJ0KCk7XG4gICAgICAgIC8vIEVuZGVkIGNvbnRhaW5lciBkcmFnIGZvciAke2VuZENvbnRhaW5lck5vZGVzLmxlbmd0aH0gbm9kZXNcbiAgICAgICAgYnJlYWs7XG4gICAgICAgIFxuICAgICAgY2FzZSAndXBkYXRlTm9kZVBvc2l0aW9ucyc6XG4gICAgICAgIC8vIFJlZHVjZWQgZnJlcXVlbmN5IHVwZGF0ZU5vZGVQb3NpdGlvbnMgbG9nZ2luZ1xuICAgICAgICBpZiAoTWF0aC5yYW5kb20oKSA8IDAuMSkge1xuICAgICAgICAgIC8vIFJlY2VpdmVkIHVwZGF0ZU5vZGVQb3NpdGlvbnM6ICR7dXBkYXRlZE5vZGVzPy5sZW5ndGggfHwgMH0gbm9kZXNcbiAgICAgICAgfVxuICAgICAgICBcbiAgICAgICAgaWYgKCFzaW11bGF0aW9uKSB7XG4gICAgICAgICAgLy8gTm8gc2ltdWxhdGlvbiBhdmFpbGFibGVcbiAgICAgICAgICByZXR1cm47XG4gICAgICAgIH1cbiAgICAgICAgXG4gICAgICAgIGlmICghdXBkYXRlZE5vZGVzKSB7XG4gICAgICAgICAgLy8gTm8gdXBkYXRlZE5vZGVzIHByb3ZpZGVkXG4gICAgICAgICAgcG9zdE1lc3NhZ2Uoe1xuICAgICAgICAgICAgdHlwZTogJ2Vycm9yJyxcbiAgICAgICAgICAgIGVycm9yOiAnTm8gYWN0aXZlIHNpbXVsYXRpb24gb3IgbWlzc2luZyBub2RlIGRhdGEnXG4gICAgICAgICAgfSBhcyBXb3JrZXJSZXNwb25zZSk7XG4gICAgICAgICAgcmV0dXJuO1xuICAgICAgICB9XG4gICAgICAgIFxuICAgICAgICAvLyBVcGRhdGUgZml4ZWQgcG9zaXRpb25zIGZvciBkcmFnZ2VkIG5vZGVzXG4gICAgICAgIGNvbnN0IHNpbU5vZGVzID0gc2ltdWxhdGlvbi5ub2RlcygpO1xuICAgICAgICAvLyBSZWR1Y2VkIGZyZXF1ZW5jeSBzaW11bGF0aW9uIHN0YXR1cyBsb2dnaW5nXG4gICAgICAgIGlmIChNYXRoLnJhbmRvbSgpIDwgMC4xKSB7XG4gICAgICAgICAgLy8gU2ltdWxhdGlvbiBoYXMgJHtzaW1Ob2Rlcy5sZW5ndGh9IG5vZGVzXG4gICAgICAgIH1cbiAgICAgICAgXG4gICAgICAgIHVwZGF0ZWROb2Rlcy5mb3JFYWNoKCh7IGlkLCBmeCwgZnkgfSkgPT4ge1xuICAgICAgICAgIGNvbnN0IG5vZGUgPSBzaW1Ob2Rlcy5maW5kKG4gPT4gbi5pZCA9PT0gaWQpO1xuICAgICAgICAgIGlmIChub2RlKSB7XG4gICAgICAgICAgICAvLyBSZWR1Y2VkIGZyZXF1ZW5jeSBub2RlIHVwZGF0ZSBsb2dnaW5nXG4gICAgICAgICAgICBpZiAoTWF0aC5yYW5kb20oKSA8IDAuMDUpIHtcbiAgICAgICAgICAgICAgLy8gVXBkYXRpbmcgbm9kZSAke2lkfTogZng9JHtmeH0sIGZ5PSR7Znl9XG4gICAgICAgICAgICB9XG4gICAgICAgICAgICBub2RlLmZ4ID0gZng7XG4gICAgICAgICAgICBub2RlLmZ5ID0gZnk7XG4gICAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICAgIC8vIE5vZGUgJHtpZH0gbm90IGZvdW5kIGluIHNpbXVsYXRpb25cbiAgICAgICAgICB9XG4gICAgICAgIH0pO1xuICAgICAgICBcbiAgICAgICAgLy8gR2VudGxlIHJlLXJlbGF4YXRpb24gYWZ0ZXIgZHJhZyBvcGVyYXRpb25zXG4gICAgICAgIC8vIFJlc3RhcnRpbmcgc2ltdWxhdGlvbiB3aXRoIGFscGhhPTAuMDVcbiAgICAgICAgc2ltdWxhdGlvbi5hbHBoYSgwLjA1KS5yZXN0YXJ0KCk7XG4gICAgICAgIGJyZWFrO1xuICAgICAgICBcbiAgICAgIGNhc2UgJ3JlaGVhdCc6XG4gICAgICAgIGlmICghc2ltdWxhdGlvbikgcmV0dXJuO1xuICAgICAgICBcbiAgICAgICAgY29uc3QgcmVoZWF0aW5nQWxwaGEgPSBhbHBoYSB8fCAwLjE7XG4gICAgICAgIHNpbXVsYXRpb24uYWxwaGEocmVoZWF0aW5nQWxwaGEpLnJlc3RhcnQoKTtcbiAgICAgICAgYnJlYWs7XG4gICAgICAgIFxuICAgICAgY2FzZSAndXBkYXRlUGh5c2ljcyc6XG4gICAgICAgIGlmICghY29uZmlnKSByZXR1cm47XG4gICAgICAgIFxuICAgICAgICBjdXJyZW50UGh5c2ljc0NvbmZpZyA9IGNvbmZpZztcbiAgICAgICAgXG4gICAgICAgIGlmIChzaW11bGF0aW9uKSB7XG4gICAgICAgICAgYXBwbHlQaHlzaWNzQ29uZmlnVG9TaW11bGF0aW9uKHNpbXVsYXRpb24sIGN1cnJlbnRQaHlzaWNzQ29uZmlnKTtcblxuICAgICAgICAgIGlmIChwaHlzaWNzQ29vbGRvd25UaW1lcikge1xuICAgICAgICAgICAgY2xlYXJUaW1lb3V0KHBoeXNpY3NDb29sZG93blRpbWVyKTtcbiAgICAgICAgICAgIHBoeXNpY3NDb29sZG93blRpbWVyID0gbnVsbDtcbiAgICAgICAgICB9XG5cbiAgICAgICAgICBzaW11bGF0aW9uLmFscGhhVGFyZ2V0KDAuMykucmVzdGFydCgpO1xuXG4gICAgICAgICAgcGh5c2ljc0Nvb2xkb3duVGltZXIgPSBzZXRUaW1lb3V0KCgpID0+IHtcbiAgICAgICAgICAgIGlmIChzaW11bGF0aW9uKSB7XG4gICAgICAgICAgICAgIHNpbXVsYXRpb24uYWxwaGFUYXJnZXQoMCk7XG4gICAgICAgICAgICB9XG4gICAgICAgICAgICBwaHlzaWNzQ29vbGRvd25UaW1lciA9IG51bGw7XG4gICAgICAgICAgfSwgMTUwMCk7XG4gICAgICAgIH1cbiAgICAgICAgYnJlYWs7XG4gICAgICAgIFxuICAgICAgY2FzZSAnc3RvcCc6XG4gICAgICAgIGlmIChzaW11bGF0aW9uKSB7XG4gICAgICAgICAgc2ltdWxhdGlvbi5vbigndGljaycsIG51bGwpO1xuICAgICAgICAgIHNpbXVsYXRpb24ub24oJ2VuZCcsIG51bGwpO1xuICAgICAgICAgIHNpbXVsYXRpb24uc3RvcCgpO1xuICAgICAgICAgIHNpbXVsYXRpb24gPSBudWxsO1xuICAgICAgICB9XG4gICAgICAgIGlmIChwaHlzaWNzQ29vbGRvd25UaW1lcikge1xuICAgICAgICAgIGNsZWFyVGltZW91dChwaHlzaWNzQ29vbGRvd25UaW1lcik7XG4gICAgICAgICAgcGh5c2ljc0Nvb2xkb3duVGltZXIgPSBudWxsO1xuICAgICAgICB9XG4gICAgICAgIGJyZWFrO1xuICAgICAgICBcbiAgICAgIGRlZmF1bHQ6XG4gICAgICAgIHBvc3RNZXNzYWdlKHtcbiAgICAgICAgICB0eXBlOiAnZXJyb3InLFxuICAgICAgICAgIGVycm9yOiBgVW5rbm93biBtZXNzYWdlIHR5cGU6ICR7dHlwZX1gXG4gICAgICAgIH0gYXMgV29ya2VyUmVzcG9uc2UpO1xuICAgIH1cbiAgfSBjYXRjaCAoZXJyb3IpIHtcbiAgICBwb3N0TWVzc2FnZSh7XG4gICAgICB0eXBlOiAnZXJyb3InLFxuICAgICAgZXJyb3I6IGVycm9yIGluc3RhbmNlb2YgRXJyb3IgPyBlcnJvci5tZXNzYWdlIDogJ1Vua25vd24gZXJyb3IgaW4gd29ya2VyJ1xuICAgIH0gYXMgV29ya2VyUmVzcG9uc2UpO1xuICB9XG59O1xuXG5mdW5jdGlvbiBmaW5kTm9kZUJ5SWQobm9kZXM6IGFueVtdLCBpZDogYW55KTogYW55IHtcbiAgZm9yIChsZXQgaSA9IDA7IGkgPCBub2Rlcy5sZW5ndGg7IGkgKz0gMSkge1xuICAgIGNvbnN0IGNhbmRpZGF0ZSA9IG5vZGVzW2ldO1xuICAgIGlmIChjYW5kaWRhdGUgJiYgY2FuZGlkYXRlLmlkID09PSBpZCkge1xuICAgICAgcmV0dXJuIGNhbmRpZGF0ZTtcbiAgICB9XG4gIH1cbiAgcmV0dXJuIHVuZGVmaW5lZDtcbn1cbiJdLAogICJtYXBwaW5ncyI6ICI7OztBQUFBLE1BQUksT0FBTyxFQUFDLE9BQU8sTUFBTTtBQUFBLEVBQUMsRUFBQztBQUUzQixXQUFTLFdBQVc7QUFDbEIsYUFBUyxJQUFJLEdBQUcsSUFBSSxVQUFVLFFBQVEsSUFBSSxDQUFDLEdBQUcsR0FBRyxJQUFJLEdBQUcsRUFBRSxHQUFHO0FBQzNELFVBQUksRUFBRSxJQUFJLFVBQVUsQ0FBQyxJQUFJLE9BQVEsS0FBSyxLQUFNLFFBQVEsS0FBSyxDQUFDLEVBQUcsT0FBTSxJQUFJLE1BQU0sbUJBQW1CLENBQUM7QUFDakcsUUFBRSxDQUFDLElBQUksQ0FBQztBQUFBLElBQ1Y7QUFDQSxXQUFPLElBQUks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Q0Usc0JBQVk7QUFBQSxZQUNWLE1BQU07QUFBQSxZQUNOLE9BQU8seUJBQXlCTCxLQUFJO0FBQUEsVUFDdEMsQ0FBbUI7QUFBQSxNQUN2QjtBQUFBLElBQ0YsU0FBUyxPQUFPO0FBQ2Qsa0JBQVk7QUFBQSxRQUNWLE1BQU07QUFBQSxRQUNOLE9BQU8saUJBQWlCLFFBQVEsTUFBTSxVQUFVO0FBQUEsTUFDbEQsQ0FBbUI7QUFBQSxJQUNyQjtBQUFBLEVBQ0Y7QUFFQSxXQUFTLGFBQWEsT0FBY0csS0FBYztBQUNoRCxhQUFTLElBQUksR0FBRyxJQUFJLE1BQU0sUUFBUSxLQUFLLEdBQUc7QUFDeEMsWUFBTSxZQUFZLE1BQU0sQ0FBQztBQUN6QixVQUFJLGFBQWEsVUFBVSxPQUFPQSxLQUFJO0FBQ3BDLGVBQU87QUFBQSxNQUNUO0FBQUEsSUFDRjtBQUNBLFdBQU87QUFBQSxFQUNUOyIsCiAgIm5hbWVzIjogWyJ0eXBlIiwgImMiLCAiZG9jdW1lbnQiLCAibSIsICJ4IiwgIm0iLCAibSIsICJkYXR1bSIsICJ4IiwgIm0iLCAic2VsZWN0aW9uIiwgIm0iLCAibSIsICJhIiwgIm0iLCAibSIsICJtIiwgImNyZWF0ZSIsICJjcmVhdGUiLCAicGFyc2VUeXBlbmFtZXMiLCAibSIsICJ0eXBlIiwgIndpbmRvdyIsICJkaXNwYXRjaF9kZWZhdWx0IiwgIm0iLCAiZGlzcGF0Y2hfZGVmYXVsdCIsICJtIiwgImEiLCAibWluIiwgIm1heCIsICJjb25zdGFudF9kZWZhdWx0IiwgIngiLCAiYSIsICJ5IiwgInkiLCAiYSIsICJjb25zdGFudF9kZWZhdWx0IiwgInkiLCAiY29sb3IiLCAicmdiIiwgInN0YXJ0IiwgImEiLCAiYSIsICJpIiwgImEiLCAiYyIsICJtIiwgImEiLCAibm93IiwgImlkIiwgImluZGV4IiwgImdldCIsICJzZXQiLCAic2VsZiIsICJzdGFydCIsICJlbXB0eSIsICJpbnRlcnJ1cHRfZGVmYXVsdCIsICJpZCIsICJzZXQiLCAiZ2V0IiwgInRyYW5zaXRpb24iLCAiYSIsICJjIiwgImF0dHJSZW1vdmUiLCAiYXR0clJlbW92ZU5TIiwgImF0dHJDb25zdGFudCIsICJhdHRyQ29uc3RhbnROUyIsICJhdHRyRnVuY3Rpb24iLCAiYXR0ckZ1bmN0aW9uTlMiLCAiYXR0cl9kZWZhdWx0IiwgImlkIiwgImdldCIsICJpZCIsICJzZXQiLCAiZ2V0IiwgImlkIiwgInNldCIsICJnZXQiLCAiaWQiLCAic2V0IiwgImZpbHRlcl9kZWZhdWx0IiwgIm0iLCAibWVyZ2VfZGVmYXVsdCIsICJ0cmFuc2l0aW9uIiwgIm0iLCAiaWQiLCAic2V0IiwgIm9uX2RlZmF1bHQiLCAiZ2V0IiwgImlkIiwgInJlbW92ZV9kZWZhdWx0IiwgInNlbGVjdF9kZWZhdWx0IiwgImlkIiwgIm0iLCAiZ2V0IiwgInNlbGVjdEFsbF9kZWZhdWx0IiwgImlkIiwgIm0iLCAiY2hpbGRyZW4iLCAiaW5oZXJpdCIsICJnZXQiLCAiU2VsZWN0aW9uIiwgInNlbGVjdGlvbl9kZWZhdWx0IiwgInN0eWxlUmVtb3ZlIiwgInN0eWxlQ29uc3RhbnQiLCAic3R5bGVGdW5jdGlvbiIsICJpZCIsICJyZW1vdmUiLCAic2V0IiwgInN0eWxlX2RlZmF1bHQiLCAidGV4dENvbnN0YW50IiwgInRleHRGdW5jdGlvbiIsICJ0ZXh0X2RlZmF1bHQiLCAibSIsICJpbmhlcml0IiwgImdldCIsICJpZCIsICJzZXQiLCAiaWQiLCAic2VsZWN0X2RlZmF1bHQiLCAic2VsZWN0QWxsX2RlZmF1bHQiLCAiZmlsdGVyX2RlZmF1bHQiLCAibWVyZ2VfZGVmYXVsdCIsICJzZWxlY3Rpb25fZGVmYXVsdCIsICJvbl9kZWZhdWx0IiwgImF0dHJfZGVmYXVsdCIsICJzdHlsZV9kZWZhdWx0IiwgInRleHRfZGVmYXVsdCIsICJyZW1vdmVfZGVmYXVsdCIsICJpZCIsICJ0cmFuc2l0aW9uX2RlZmF1bHQiLCAibSIsICJpbnRlcnJ1cHRfZGVmYXVsdCIsICJ0cmFuc2l0aW9uX2RlZmF1bHQiLCAieCIsICJ5IiwgIngiLCAieSIsICJ4IiwgInkiLCAieCIsICJ5IiwgImRhdGFfZGVmYXVsdCIsICJ4IiwgInkiLCAieDIiLCAieTIiLCAieDMiLCAieTMiLCAicmVtb3ZlX2RlZmF1bHQiLCAieCIsICJ5IiwgInNpemVfZGVmYXVsdCIsICJ4IiwgInkiLCAiZGF0YV9kZWZhdWx0IiwgInJlbW92ZV9kZWZhdWx0IiwgInNpemVfZGVmYXVsdCIsICJjb25zdGFudF9kZWZhdWx0IiwgIngiLCAiY29uc3RhbnRfZGVmYXVsdCIsICJ4IiwgInkiLCAiZmluZCIsICJpZCIsICJjb25zdGFudF9kZWZhdWx0IiwgIngiLCAieSIsICJtIiwgImkiLCAieCIsICJ5IiwgInNpbXVsYXRpb24iLCAiY29uc3RhbnRfZGVmYXVsdCIsICJ4IiwgInkiLCAibm9kZSIsICJzdHJlbmd0aCIsICJjIiwgIngyIiwgInhfZGVmYXVsdCIsICJ4IiwgImNvbnN0YW50X2RlZmF1bHQiLCAieV9kZWZhdWx0IiwgInkiLCAiY29uc3RhbnRfZGVmYXVsdCIsICJ4IiwgInkiLCAiaWRlbnRpdHkiLCAiaWQiLCAieCIsICJ5IiwgInhfZGVmYXVsdCIsICJ5X2RlZmF1bHQiLCAidHlwZSIsICJzaW1Ob2RlcyIsICJub2RlcyIsICJpZCIsICJmeCIsICJmeSJdCn0K
