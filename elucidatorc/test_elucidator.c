#include <stdio.h>
#include "elucidator.h"

int main() {
    struct DesignationHandle hdl = get_designation_from_text("foo", "bar: u32");
    struct DesignationHandle hdl2 = get_designation_from_text(
        "baz",
        "age: u8, net_worth_usd: i32, name: string"
    );
    print_designation(&hdl);
    print_designation(&hdl2);
}
