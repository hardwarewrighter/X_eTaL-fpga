// Board adapter: UART characters to K/R handshake, digit strobes to text.
// The calculator and UART state machines are emitted X_eTaL cores.
module hex1000_top(input clk, input rst_n, input uart_rx, output uart_tx, output led);
  reg [7:0] startup = 0;
  reg [1:0] reset_pipe = 3, rx_pipe = 3;
  reg tx_pin = 1;
  reg [23:0] heartbeat = 0;
  reg [3:0] clock_divider = 0;
  reg machine_clk = 0;
  wire reset = reset_pipe[1] | ~(&startup);
  wire [15:0] tx_line, rx_byte, rx_valid, tx_busy;
  wire [15:0] tx_command;
  wire [15:0] machine_r;
  wire [5:0] machine_o;
  reg [15:0] r_meta = 0, r_sync = 0, r_old = 0;
  reg [5:0] o_meta = 0, o_sync = 0;
  reg [4:0] keys [0:31];
  reg [4:0] key_write = 0, key_read = 0;
  reg [5:0] key_count = 0;
  reg key_active = 0;
  reg [4:0] key_code = 0;
  reg [1:0] key_active_sync = 0;
  reg [4:0] key_code_meta = 0, key_code_sync = 0;
  reg [4:0] digits [0:7];
  reg [63:0] pending_frame = 0, send_frame = 0;
  reg frame_pending = 0, sending = 0, tx_wait = 0;
  reg [3:0] tx_position = 0;
  integer i;

  function [5:0] decode_key;
    input [7:0] character;
    begin
      if (character >= "0" && character <= "9") decode_key = {1'b1, 5'b0} + character - "0";
      else if (character >= "A" && character <= "F") decode_key = 6'd42 + character - "A";
      else if (character >= "a" && character <= "f") decode_key = 6'd42 + character - "a";
      else case (character)
        "+": decode_key = 48;
        "-": decode_key = 49;
        "*": decode_key = 50;
        "=", 8'd13, 8'd10: decode_key = 51;
        "X", "x": decode_key = 52;
        default: decode_key = 0;
      endcase
    end
  endfunction
  function [7:0] digit_text;
    input [4:0] digit;
    begin
      if (!digit[4]) digit_text = " ";
      else if (digit[3:0] < 10) digit_text = "0" + digit[3:0];
      else digit_text = "A" + digit[3:0] - 10;
    end
  endfunction

  wire [5:0] decoded = decode_key(rx_byte[7:0]);
  wire push_key = rx_valid[0] && decoded[5] && key_count < 32;
  wire pop_key = !key_active && key_count != 0 && r_sync[15];
  wire new_frame = r_sync[0] && !r_old[0];
  wire tx_start = sending && !tx_wait && !tx_busy[0];
  wire [7:0] tx_byte = tx_position == 8 ? 8'd13 : send_frame[(7-tx_position)*8 +: 8];
  assign tx_command = tx_start ? {8'b0, tx_byte} : 16'hffff;
  wire [4:0] keyboard = (key_active_sync[1] && machine_r[8 + key_code_sync[4:2]])
                        ? (5'b1 << key_code_sync[1:0]) : 5'b0;
  assign uart_tx = tx_pin;
  assign led = ~heartbeat[23];

  uart serial_core(.clk(clk), .rst(reset), .in_rxline({15'b0, rx_pipe[1]}), .in_c(tx_command),
                   .out_line(tx_line), .out_rxbyte(rx_byte), .out_valid(rx_valid), .out_busy(tx_busy));
  hex1000 machine_core(.clk(machine_clk), .rst(reset), .in_k(keyboard), .out_o(machine_o), .out_r(machine_r));

  // Keep the machine clock running during its synchronous reset.
  always @(posedge clk) begin
    if (clock_divider == 13) begin clock_divider <= 0; machine_clk <= ~machine_clk; end
    else clock_divider <= clock_divider + 1'b1;
  end
  always @(posedge machine_clk) begin
    if (reset) begin key_active_sync <= 0; key_code_meta <= 0; key_code_sync <= 0; end
    else begin key_active_sync <= {key_active_sync[0], key_active}; key_code_meta <= key_code; key_code_sync <= key_code_meta; end
  end
  always @(posedge clk) begin
    reset_pipe <= {reset_pipe[0], !rst_n};
    rx_pipe <= {rx_pipe[0], uart_rx};
    if (!rst_n) startup <= 0;
    else if (!(&startup)) startup <= startup + 1'b1;
    if (reset) begin
      tx_pin <= 1; heartbeat <= 0;
      r_meta <= 0; r_sync <= 0; r_old <= 0; o_meta <= 0; o_sync <= 0;
      key_write <= 0; key_read <= 0; key_count <= 0; key_active <= 0; key_code <= 0;
      frame_pending <= 0; pending_frame <= 0; sending <= 0; send_frame <= 0; tx_wait <= 0; tx_position <= 0;
      for (i = 0; i < 8; i = i + 1) digits[i] <= 0;
    end else begin
      tx_pin <= tx_line[0]; heartbeat <= heartbeat + 1'b1;
      r_meta <= machine_r; r_sync <= r_meta; r_old <= r_sync;
      o_meta <= machine_o; o_sync <= o_meta;
      if (push_key) begin keys[key_write] <= decoded[4:0]; key_write <= key_write + 1'b1; end
      if (pop_key) begin key_code <= keys[key_read]; key_read <= key_read + 1'b1; key_active <= 1; end
      if (key_active && !r_sync[15]) key_active <= 0;
      case ({push_key, pop_key})
        2'b10: key_count <= key_count + 1'b1;
        2'b01: key_count <= key_count - 1'b1;
      endcase
      for (i = 0; i < 8; i = i + 1)
        if (r_sync[i] && !r_old[i]) digits[i] <= o_sync[4:0];
      if (new_frame) begin
        pending_frame <= {digit_text(digits[7]), digit_text(digits[6]), digit_text(digits[5]), digit_text(digits[4]),
                          digit_text(digits[3]), digit_text(digits[2]), digit_text(digits[1]), digit_text(o_sync[4:0])};
        frame_pending <= 1;
      end
      if (frame_pending && !sending) begin
        send_frame <= pending_frame; sending <= 1; tx_position <= 0; tx_wait <= 0;
        frame_pending <= new_frame;
      end
      if (tx_start) tx_wait <= 1;
      else if (sending && tx_wait && tx_busy[0]) begin
        tx_wait <= 0;
        if (tx_position == 8) sending <= 0;
        else tx_position <= tx_position + 1'b1;
      end
    end
  end
endmodule
