// Board wiring and timing only; arithmetic and UART come from emitted XIR.
module probe_top(input clk, input rst_n, input uart_rx, output uart_tx, output led);
  reg [7:0] startup = 0;
  reg [1:0] reset_pipe = 2'b11;
  reg [1:0] rx_pipe = 2'b11;
  reg [15:0] divider = 0;
  reg tx_pin = 1;
  wire reset = reset_pipe[1] | ~(&startup);
  wire [7:0] heartbeat;
  wire [15:0] line;
  wire [7:0] query_count;
  always @(posedge clk) begin
    reset_pipe <= {reset_pipe[0], !rst_n};
    rx_pipe <= {rx_pipe[0], uart_rx};
    if (reset) tx_pin <= 1;
    else tx_pin <= line[0];
    if (!rst_n) startup <= 0;
    else if (!(&startup)) startup <= startup + 1'b1;
    if (reset) divider <= 0;
    else divider <= divider + 1'b1;
  end
  counter blink_core(.clk(clk), .rst(reset), .in_e({7'b0, &divider}), .out_s(heartbeat));
  probe probe_core(.clk(clk), .rst(reset), .in_rxline({15'b0, rx_pipe[1]}),
                   .out_line(line), .out_count(query_count));
  assign uart_tx = tx_pin;
  assign led = ~heartbeat[7];
endmodule
