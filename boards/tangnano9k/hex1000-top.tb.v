`timescale 1ns/1ps
module tb;
reg clk=0,rst_n=1,uart_rx=1;
wire uart_tx,led;
hex1000_top dut(clk,rst_n,uart_rx,uart_tx,led);
always #18.5185 clk=~clk;
reg[7:0] value; integer bit_index,frames=0;
reg[63:0] last_frame=0,partial_frame=0;
always @(negedge uart_tx) begin
 #13020;
 for(bit_index=0;bit_index<8;bit_index=bit_index+1) begin value[bit_index]=uart_tx;#8680;end
 if(uart_tx!==1)$fatal(1,"UART stop");
 if(value==13)begin last_frame=partial_frame;frames=frames+1;$display("FRAME %s",last_frame);end
 else partial_frame=(partial_frame<<8)|value;
end
task send;input[7:0] data;integer i;begin
 uart_rx=0;#8680;for(i=0;i<8;i=i+1)begin uart_rx=data[i];#8680;end
 uart_rx=1;#258680;
end endtask
initial begin
 #3000000;
 send("X");send("1");send("2");send("+");send("3");send("4");send("=");
 #5000000;
 if(last_frame!=="      46")$fatal(1,"wrong addition result");
 send("X");send("A");send("*");send("B");send("=");
 #5000000;
 if(last_frame!=="      6E")$fatal(1,"wrong multiplication result");
 send("X");send("C");send("0");send("D");send("E");send("+");
 send("B");send("E");send("E");send("F");send("=");
 #5000000;
 if(last_frame!=="   17FCD")$fatal(1,"wrong multi-digit result");
 $display("PASS HEX1000 serial calculations: 46, 6E, 17FCD");$finish;
end
initial begin #100000000; $fatal(1,"calculator simulation timeout"); end
endmodule
