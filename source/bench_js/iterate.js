/*
> time node source/bench_js/iterate.js
*/

var x = 0
for (var i = 0; i < 100000000; i += 1) {
    x = Math.sqrt(x+1)
}
console.log("x " + x)
