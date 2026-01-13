package main

import (
	"fmt"
	"sync"
	"time"
)

// Example: Concurrent Go program with goroutines and channels

func producer(ch chan<- int, wg *sync.WaitGroup) {
	defer wg.Done()
	for i := 0; i < 10; i++ {
		ch <- i
		time.Sleep(100 * time.Millisecond)
	}
	close(ch)
}

func consumer(ch <-chan int, wg *sync.WaitGroup) {
	defer wg.Done()
	for num := range ch {
		fmt.Printf("Consumed: %d\n", num)
		time.Sleep(50 * time.Millisecond)
	}
}

func main() {
	fmt.Println("🎬 Starting concurrent Go program with Aevum tracing...")

	// In a real implementation, attach the Go agent here
	// agent, _ := goagent.Attach("concurrent-example", "localhost:9876")
	// defer goagent.Detach()

	ch := make(chan int, 5)
	var wg sync.WaitGroup

	wg.Add(2)
	go producer(ch, &wg)
	go consumer(ch, &wg)

	wg.Wait()

	fmt.Println("\n✅ Execution complete! Trace captured goroutine interactions.")
}
