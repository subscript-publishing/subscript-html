"use strict";

exports.createElement = function (tag) {
    return function() {
        return document.createElement(tag);
    };
};

exports.setAttribute = function (key) {
    return function(value) {
        return function(doc) {
            return function() {
                return doc.setAttribute(key, value);
            }
        };
    };
};


exports.setAttribute = function (key) {
    return function(value) {
        return function(doc) {
            return function() {
                return doc.setAttribute(key, value);
            }
        };
    };
};




