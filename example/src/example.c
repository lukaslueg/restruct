typedef struct {
    char c;
    short h;
    long l;
    char x;
    int i;
    char x2;
    long long ll;
    unsigned char uc;
    unsigned short us;
    unsigned long ul;
    char x3;
    unsigned int ui;
    char x4;
    unsigned long long ull;
    char s[3];
} turtle;

typedef struct {
    char c;
    unsigned long long ull;
    short h;
} lower_turtle;

typedef struct {
    int i;
    lower_turtle t[2];
} lowest_turtle;

static turtle t = { 100, -32000, -200000000, 127, -1000000000, 100, 10000000000, 128, 32000, 400000000, 3, 300000000, 4, 100000000000, {1, 2, 3}};

static lowest_turtle lt = { -1, { { 100, 127, 128}, { 100, 10000000000, -32000 }} };

turtle *reach_turtle() {
    return &t;
}

lowest_turtle *reach_lowest_turtle() {
    return &lt;
}
