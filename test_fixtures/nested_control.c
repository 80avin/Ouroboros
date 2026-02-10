int nested(int x, int y) {
    if (x > 0) {
        if (y > 0) {
            return x + y;
        } else {
            return x - y;
        }
    } else {
        return 0;
    }
}

int main() {
    return nested(5, 3);
}
