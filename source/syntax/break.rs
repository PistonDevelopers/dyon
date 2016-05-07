fn main() {
    for i := 0; i < 1000; i += 1 {
        print("Do you want to quit? (y/n): ")
        answer := trim_right(read_line())
        if answer == "y" {
            break
        }
    }
}
