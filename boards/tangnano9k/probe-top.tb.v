`timescale 1ns/1ps
// Exercise board timing at 27 MHz and 115200 baud, before and after synthesis.
module tb;
  reg clk = 0, rst_n = 1, uart_rx = 1;
  wire uart_tx, led;
  probe_top dut(clk, rst_n, uart_rx, uart_tx, led);
  always #18.5185 clk = ~clk;
  integer received = 0, bit_index;
  reg [7:0] data, expected;
  always @(negedge uart_tx) begin
    #13020;
    for (bit_index = 0; bit_index < 8; bit_index = bit_index + 1) begin
      data[bit_index] = uart_tx;
      #8680;
    end
    if (uart_tx !== 1) $fatal(1, "missing stop bit");
    case (received)
      0: expected = 0;
      1: expected = 255;
      2: expected = 65;
      3: expected = 124;
      4: expected = 85;
      5: expected = 170;
      default: $fatal(1, "extra UART frame (pin glitch)");
    endcase
    if (data !== expected) $fatal(1, "wrong UART response: %d expected %d", data, expected);
    received = received + 1;
  end
  task send;
    input [7:0] value;
    integer i;
    begin
      uart_rx = 0;
      #8680;
      for (i = 0; i < 8; i = i + 1) begin
        uart_rx = value[i];
        #8680;
      end
      uart_rx = 1;
      #158680;
    end
  endtask
  initial begin
    #100000;
    send(0); send(255); send(65); send(63); send(85); send(170);
    #100000;
    if (received != 6) $fatal(1, "missing UART replies");
    $display("PASS: board startup, 115200 UART echo/query, no extra frames");
    $finish;
  end
endmodule
