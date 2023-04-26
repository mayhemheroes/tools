function hello_world_test(list) {
    let x = 5;
    if (list.length > 0) {
        x = 10;
    }

    let y = 15;

    for (let i = 0; i < 10; i++) {
        y = y + 1;
    }

    console.log(x + y);

    return x + y;
}

function main() {
    let list = [1, 2, 3, 4, 5];
    let result = hello_world_test(list);
    console.log(result);
}