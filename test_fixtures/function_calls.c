int add(int a, int b) {
    return a + b;
}

int mult(int a, int b) {
    return a * b;
}

int main() {
    return add(mult(2, 3), 4);
}
